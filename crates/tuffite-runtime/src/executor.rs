//! Safe access to Chromium's process-wide task scheduler.

use std::ffi::c_void;
#[cfg(feature = "async")]
use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
#[cfg(feature = "async")]
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
#[cfg(feature = "async")]
use std::task::{Context, Poll, Waker};

use tuffite_abi::Status;

use crate::abi::NativeHost;

/// Scheduling priority forwarded to Chromium.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TaskPriority {
    /// Deferrable work such as maintenance and non-critical persistence.
    BestEffort,
    /// Work whose result will become visible without blocking an interaction.
    #[default]
    UserVisible,
    /// Work required to complete an immediate user action.
    UserBlocking,
}

impl From<TaskPriority> for tuffite_abi::TaskPriority {
    fn from(value: TaskPriority) -> Self {
        match value {
            TaskPriority::BestEffort => Self::BestEffort,
            TaskPriority::UserVisible => Self::UserVisible,
            TaskPriority::UserBlocking => Self::UserBlocking,
        }
    }
}

/// Failure while submitting work to Chromium.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ExecutorError {
    /// The application is stopping and no longer accepts new work.
    #[error("Chromium executor is shutting down")]
    ShuttingDown,
    /// Chromium rejected the task submission.
    #[error("Chromium rejected the task")]
    Rejected,
}

/// Failure reported by an awaited Chromium task.
#[cfg(feature = "async")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ChromiumTaskError {
    /// The submitted closure panicked while Chromium was running it.
    #[error("Chromium task panicked")]
    Panicked,
}

#[cfg(feature = "async")]
struct CompletionState<T> {
    result: Option<Result<T, ChromiumTaskError>>,
    finished: bool,
    waker: Option<Waker>,
}

#[cfg(feature = "async")]
struct Completion<T> {
    state: Mutex<CompletionState<T>>,
}

#[cfg(feature = "async")]
impl<T> Completion<T> {
    fn new() -> Self {
        Self {
            state: Mutex::new(CompletionState {
                result: None,
                finished: false,
                waker: None,
            }),
        }
    }

    fn finish(&self, result: Result<T, ChromiumTaskError>) {
        let waker = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.result = Some(result);
            state.finished = true;
            state.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

/// Awaitable result of one finite closure scheduled on Chromium's thread pool.
///
/// Dropping the handle does not cancel work already accepted by Chromium.
#[cfg(feature = "async")]
#[must_use = "Chromium tasks do nothing with their result unless awaited"]
pub struct ChromiumTask<T> {
    completion: Arc<Completion<T>>,
}

#[cfg(feature = "async")]
impl<T> std::fmt::Debug for ChromiumTask<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ChromiumTask")
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "async")]
impl<T> Future for ChromiumTask<T> {
    type Output = Result<T, ChromiumTaskError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self
            .completion
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(result) = state.result.take() {
            return Poll::Ready(result);
        }
        assert!(!state.finished, "ChromiumTask polled after completion");
        if state
            .waker
            .as_ref()
            .is_none_or(|waker| !waker.will_wake(context.waker()))
        {
            state.waker = Some(context.waker().clone());
        }
        Poll::Pending
    }
}

struct State {
    accepting: AtomicBool,
    pending: Mutex<usize>,
    drained: Condvar,
}

struct TaskEnvelope {
    task: Option<Box<dyn FnOnce() + Send + 'static>>,
    state: Arc<State>,
}

impl Drop for TaskEnvelope {
    fn drop(&mut self) {
        let mut pending = self
            .state
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *pending -= 1;
        if *pending == 0 {
            self.state.drained.notify_all();
        }
    }
}

unsafe extern "C" fn run_task(task_data: *mut c_void) {
    // SAFETY: `post` transfers one live TaskEnvelope allocation to Framework.
    let envelope = unsafe { &mut *task_data.cast::<TaskEnvelope>() };
    if let Some(task) = envelope.task.take() {
        let _ = catch_unwind(AssertUnwindSafe(task));
    }
}

unsafe extern "C" fn destroy_task(task_data: *mut c_void) {
    // SAFETY: Framework invokes destroy exactly once for an accepted task.
    drop(unsafe { Box::from_raw(task_data.cast::<TaskEnvelope>()) });
}

/// Cloneable, thread-safe handle to Chromium's process-wide thread pool.
#[derive(Clone)]
pub struct ChromiumExecutor {
    callbacks: NativeHost,
    state: Arc<State>,
}

impl std::fmt::Debug for ChromiumExecutor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ChromiumExecutor")
            .field("accepting", &self.state.accepting.load(Ordering::Acquire))
            .finish_non_exhaustive()
    }
}

impl ChromiumExecutor {
    pub(crate) fn new(callbacks: NativeHost) -> Self {
        Self {
            callbacks,
            state: Arc::new(State {
                accepting: AtomicBool::new(true),
                pending: Mutex::new(0),
                drained: Condvar::new(),
            }),
        }
    }

    /// Posts one finite task with Chromium scheduling semantics.
    ///
    /// Set `may_block` for filesystem or other potentially blocking calls.
    /// Accepted tasks finish before application shutdown returns.
    pub fn post<F>(
        &self,
        priority: TaskPriority,
        may_block: bool,
        task: F,
    ) -> Result<(), ExecutorError>
    where
        F: FnOnce() + Send + 'static,
    {
        let mut pending = self
            .state
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !self.state.accepting.load(Ordering::Acquire) {
            return Err(ExecutorError::ShuttingDown);
        }
        *pending += 1;
        drop(pending);

        let envelope = Box::new(TaskEnvelope {
            task: Some(Box::new(task)),
            state: Arc::clone(&self.state),
        });
        let task_data = Box::into_raw(envelope).cast::<c_void>();
        // SAFETY: The accepted ABI table stays valid until application stop.
        // On success Framework owns `task_data`; on failure ownership remains
        // here and is reconstructed immediately below.
        let status = unsafe {
            self.callbacks.post_task(
                priority.into(),
                u8::from(may_block),
                task_data,
                Some(run_task),
                Some(destroy_task),
            )
        };
        if status == Status::Ok {
            Ok(())
        } else {
            // SAFETY: A failed submission does not consume the task allocation.
            drop(unsafe { Box::from_raw(task_data.cast::<TaskEnvelope>()) });
            Err(ExecutorError::Rejected)
        }
    }

    /// Posts one finite closure and returns an awaitable result handle.
    ///
    /// This adapts Chromium scheduling to Rust `async`; it is not a Tokio
    /// runtime and does not provide Tokio's I/O, timer, or task drivers.
    #[cfg(feature = "async")]
    pub fn spawn<F, T>(
        &self,
        priority: TaskPriority,
        may_block: bool,
        task: F,
    ) -> Result<ChromiumTask<T>, ExecutorError>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let completion = Arc::new(Completion::new());
        let task_completion = Arc::clone(&completion);
        self.post(priority, may_block, move || {
            let result =
                catch_unwind(AssertUnwindSafe(task)).map_err(|_| ChromiumTaskError::Panicked);
            task_completion.finish(result);
        })?;
        Ok(ChromiumTask { completion })
    }

    pub(crate) fn close_and_wait(&self) {
        self.state.accepting.store(false, Ordering::Release);
        let pending = self
            .state
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        drop(
            self.state
                .drained
                .wait_while(pending, |pending| *pending != 0)
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tuffite_abi::{ByteSlice, Handle, RuntimeEventKind};

    struct DropMarker(Arc<AtomicBool>);

    impl Drop for DropMarker {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }

    unsafe extern "C" fn ignore_event(
        _user_data: *mut c_void,
        _runtime: Handle,
        _kind: RuntimeEventKind,
        _payload: ByteSlice,
    ) -> Status {
        Status::Ok
    }

    unsafe extern "C" fn run_immediately(
        _user_data: *mut c_void,
        _priority: tuffite_abi::TaskPriority,
        _may_block: u8,
        task_data: *mut c_void,
        run: Option<tuffite_abi::TaskRunFn>,
        destroy: Option<tuffite_abi::TaskDestroyFn>,
    ) -> Status {
        let (Some(run), Some(destroy)) = (run, destroy) else {
            return Status::InvalidArgument;
        };
        // SAFETY: The executor owns the allocation until this accepted call.
        unsafe {
            run(task_data);
            destroy(task_data);
        }
        Status::Ok
    }

    unsafe extern "C" fn reject(
        _user_data: *mut c_void,
        _priority: tuffite_abi::TaskPriority,
        _may_block: u8,
        _task_data: *mut c_void,
        _run: Option<tuffite_abi::TaskRunFn>,
        _destroy: Option<tuffite_abi::TaskDestroyFn>,
    ) -> Status {
        Status::InvalidState
    }

    fn executor(post_task: tuffite_abi::PostTaskFn) -> ChromiumExecutor {
        ChromiumExecutor::new(NativeHost(tuffite_abi::Host {
            header: tuffite_abi::Header::for_type::<tuffite_abi::Host>(),
            user_data: std::ptr::null_mut(),
            post_event: Some(ignore_event),
            post_task: Some(post_task),
            query_extension: None,
        }))
    }

    #[test]
    fn accepted_task_runs_and_drains() {
        let count = Arc::new(AtomicUsize::new(0));
        let task_count = Arc::clone(&count);
        let executor = executor(run_immediately);
        executor
            .post(TaskPriority::UserVisible, false, move || {
                task_count.fetch_add(1, Ordering::Relaxed);
            })
            .unwrap();
        executor.close_and_wait();
        assert_eq!(count.load(Ordering::Relaxed), 1);
        assert_eq!(
            executor.post(TaskPriority::BestEffort, false, || {}),
            Err(ExecutorError::ShuttingDown)
        );
    }

    #[test]
    fn rejected_task_is_reclaimed() {
        let dropped = Arc::new(AtomicBool::new(false));
        let marker = DropMarker(Arc::clone(&dropped));
        assert_eq!(
            executor(reject).post(TaskPriority::UserBlocking, true, move || drop(marker)),
            Err(ExecutorError::Rejected)
        );
        assert!(dropped.load(Ordering::Acquire));
    }

    #[cfg(feature = "async")]
    #[test]
    fn chromium_task_is_awaitable() {
        let task = executor(run_immediately)
            .spawn(TaskPriority::UserVisible, false, || 42)
            .unwrap();
        assert_eq!(futures_executor::block_on(task), Ok(42));
    }

    #[cfg(feature = "async")]
    #[test]
    fn chromium_task_reports_panics() {
        let task = executor(run_immediately)
            .spawn(TaskPriority::UserVisible, false, || panic!("expected"))
            .unwrap();
        assert_eq!(
            futures_executor::block_on(task),
            Err(ChromiumTaskError::Panicked)
        );
    }
}
