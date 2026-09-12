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

#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub(crate) fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
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
