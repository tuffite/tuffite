use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::handle::{Handle, HandleError, HandleKind, HandleTable, HandleType};

thread_local! {
    static IS_RUNTIME_WORKER: Cell<bool> = const { Cell::new(false) };
}

pub(crate) enum TaskTag {}

impl HandleType for TaskTag {
    const KIND: HandleKind = HandleKind::Task;
}

pub(crate) type InternalTaskHandle = Handle<TaskTag>;

#[derive(Clone, Default)]
pub struct CancellationToken {
    state: Arc<CancellationState>,
}

#[derive(Default)]
struct CancellationState {
    cancelled: AtomicBool,
    next_waiter: std::sync::atomic::AtomicU64,
    waiters: Mutex<std::collections::HashMap<u64, std::task::Waker>>,
}

impl CancellationToken {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub fn is_cancelled(&self) -> bool {
        self.state.cancelled.load(Ordering::Acquire)
    }

    pub(crate) fn cancel(&self) {
        let waiters = {
            let mut waiters = self
                .state
                .waiters
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.state.cancelled.store(true, Ordering::Release);
            std::mem::take(&mut *waiters)
        };
        for (_, waiter) in waiters {
            waiter.wake();
        }
    }

    pub(crate) fn cancelled(&self) -> Cancelled {
        Cancelled {
            token: self.clone(),
            id: self.state.next_waiter.fetch_add(1, Ordering::Relaxed),
        }
    }
}

pub(crate) struct Cancelled {
    token: CancellationToken,
    id: u64,
}

impl std::future::Future for Cancelled {
    type Output = ();
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<()> {
        let mut waiters = self
            .token
            .state
            .waiters
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.token.is_cancelled() {
            waiters.remove(&self.id);
            std::task::Poll::Ready(())
        } else {
            waiters.insert(self.id, cx.waker().clone());
            std::task::Poll::Pending
        }
    }
}

impl Drop for Cancelled {
    fn drop(&mut self) {
        self.token
            .state
            .waiters
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&self.id);
    }
}

struct TaskRecord {
    cancellation: CancellationToken,
    thread: Option<JoinHandle<()>>,
}

#[derive(Default)]
pub(crate) struct TaskSupervisor {
    tasks: Mutex<HandleTable<TaskRecord, TaskTag>>,
}

impl TaskSupervisor {
    pub(crate) fn spawn<F>(&self, name: &str, task: F) -> Result<InternalTaskHandle, HandleError>
    where
        F: FnOnce(CancellationToken) + Send + 'static,
    {
        let cancellation = CancellationToken::new();
        let worker_cancellation = cancellation.clone();
        let mut tasks = self
            .tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let handle = tasks.insert(TaskRecord {
            cancellation,
            thread: None,
        })?;
        let span =
            tracing::debug_span!("runtime.task", task.name = name, task.handle = ?handle.raw());
        let Ok(thread) = thread::Builder::new().name(name.to_owned()).spawn(move || {
            let _entered = span.enter();
            IS_RUNTIME_WORKER.set(true);
            task(worker_cancellation);
            IS_RUNTIME_WORKER.set(false);
        }) else {
            let _ = tasks.remove(handle);
            return Err(HandleError::Exhausted);
        };
        tasks.get_mut(handle)?.thread = Some(thread);
        Ok(handle)
    }

    pub(crate) fn cancel(&self, handle: InternalTaskHandle) -> Result<(), HandleError> {
        let tasks = self
            .tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        tasks.get(handle)?.cancellation.cancel();
        Ok(())
    }

    pub(crate) fn stop_and_join(&self) {
        let mut records = self
            .tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain();
        for record in &records {
            record.cancellation.cancel();
        }
        for record in &mut records {
            if let Some(thread) = record.thread.take() {
                let name = thread.thread().name().unwrap_or("unnamed").to_owned();
                if thread.join().is_err() {
                    tracing::error!(task.name = %name, "supervised task panicked");
                }
            }
        }
    }

    pub(crate) fn is_worker_thread() -> bool {
        IS_RUNTIME_WORKER.get()
    }
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    use std::future::Future;
    use std::task::{Context, Poll, Wake, Waker};

    struct Counter(std::sync::atomic::AtomicUsize);
    impl Wake for Counter {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn cancellation_wakes_all_live_waiters_and_removes_dropped_ones() {
        let token = CancellationToken::new();
        let counter = Arc::new(Counter(std::sync::atomic::AtomicUsize::new(0)));
        let waker = Waker::from(counter.clone());
        let mut cx = Context::from_waker(&waker);
        let mut first = Box::pin(token.cancelled());
        let mut second = Box::pin(token.cancelled());
        assert!(first.as_mut().poll(&mut cx).is_pending());
        assert!(second.as_mut().poll(&mut cx).is_pending());
        drop(second);
        assert_eq!(token.state.waiters.lock().unwrap().len(), 1);
        token.cancel();
        assert_eq!(counter.0.load(Ordering::SeqCst), 1);
        assert_eq!(first.as_mut().poll(&mut cx), Poll::Ready(()));
        assert!(token.state.waiters.lock().unwrap().is_empty());
    }
}
