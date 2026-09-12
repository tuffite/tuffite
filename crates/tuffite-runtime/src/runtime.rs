use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::handle::{HandleError, RawHandle};
use crate::supervisor::{CancellationToken, InternalTaskHandle, TaskSupervisor};

/// Observable Runtime lifecycle states.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum RuntimeState {
    /// Runtime accepts new work.
    Running = 1,
    /// Runtime is cancelling and joining supervised tasks.
    Stopping = 2,
    /// Runtime has released its supervised work.
    Stopped = 3,
}

impl RuntimeState {
    const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Running,
            2 => Self::Stopping,
            _ => Self::Stopped,
        }
    }
}

/// Errors produced by Runtime lifecycle and task operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RuntimeError {
    /// Runtime no longer accepts new work.
    #[error("runtime is not running")]
    NotRunning,
    /// Stopping from a supervised worker would self-join.
    #[error("runtime cannot stop from a supervised worker")]
    StopFromWorker,
    /// The task handle was invalid or stale.
    #[error("invalid task: {0}")]
    InvalidTask(#[source] HandleError),
}

/// Context passed to one supervised task.
#[derive(Clone)]
pub struct TaskContext {
    cancellation: CancellationToken,
}

impl TaskContext {
    /// Reports whether Runtime has requested cooperative cancellation.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }
}

/// Public pointer-free identity of a supervised Runtime task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskHandle(InternalTaskHandle);

impl TaskHandle {
    /// Returns the stable opaque handle value.
    #[must_use]
    pub const fn as_raw(self) -> RawHandle {
        self.0.raw()
    }
}

/// Rust control plane lifecycle and task supervisor.
pub struct Runtime {
    state: AtomicU8,
    lifecycle: Mutex<()>,
    supervisor: TaskSupervisor,
}

impl Runtime {
    /// Creates a running Runtime.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(RuntimeState::Running as u8),
            lifecycle: Mutex::new(()),
            supervisor: TaskSupervisor::default(),
        }
    }

    /// Returns the current lifecycle state.
    #[must_use]
    pub fn state(&self) -> RuntimeState {
        RuntimeState::from_u8(self.state.load(Ordering::Acquire))
    }

    /// Starts a named supervised task.
    pub fn spawn<F>(&self, name: &str, task: F) -> Result<TaskHandle, RuntimeError>
    where
        F: FnOnce(TaskContext) + Send + 'static,
    {
        let _lifecycle = self
            .lifecycle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.state() != RuntimeState::Running {
            return Err(RuntimeError::NotRunning);
        }
        self.supervisor
            .spawn(name, move |cancellation| {
                task(TaskContext { cancellation });
            })
            .map(TaskHandle)
            .map_err(RuntimeError::InvalidTask)
    }

    /// Requests cooperative cancellation for one task.
    pub fn cancel(&self, task: TaskHandle) -> Result<(), RuntimeError> {
        self.supervisor
            .cancel(task.0)
            .map_err(RuntimeError::InvalidTask)
    }

    /// Stops accepting work, cancels all tasks, and joins their threads.
    pub fn stop(&self) -> Result<(), RuntimeError> {
        if TaskSupervisor::is_worker_thread() {
            return Err(RuntimeError::StopFromWorker);
        }
        let lifecycle = self
            .lifecycle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if self
            .state
            .compare_exchange(
                RuntimeState::Running as u8,
                RuntimeState::Stopping as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_err()
        {
            return if self.state() == RuntimeState::Stopped {
                Ok(())
            } else {
                Err(RuntimeError::NotRunning)
            };
        }

        // No later spawn can pass the Running check. Release the lifecycle lock
        // before joining so a worker racing to spawn can observe Stopping and
        // exit instead of deadlocking behind its own join.
        drop(lifecycle);
        tracing::debug!("runtime stopping");
        self.supervisor.stop_and_join();
        self.state
            .store(RuntimeState::Stopped as u8, Ordering::Release);
        tracing::debug!("runtime stopped");
        Ok(())
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if !TaskSupervisor::is_worker_thread() {
            let _ = self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn stop_cancels_and_joins_tasks() {
        let runtime = Runtime::new();
        let exited = Arc::new(AtomicBool::new(false));
        let worker_exited = Arc::clone(&exited);
        runtime
            .spawn("test-worker", move |context| {
                while !context.is_cancelled() {
                    thread::yield_now();
                }
                worker_exited.store(true, Ordering::Release);
            })
            .unwrap();

        runtime.stop().unwrap();
        assert_eq!(runtime.state(), RuntimeState::Stopped);
        assert!(exited.load(Ordering::Acquire));
        assert_eq!(runtime.spawn("late", |_| {}), Err(RuntimeError::NotRunning));
    }

    #[test]
    fn individual_cancellation_is_observed() {
        let runtime = Runtime::new();
        let exited = Arc::new(AtomicBool::new(false));
        let worker_exited = Arc::clone(&exited);
        let handle = runtime
            .spawn("cancel-worker", move |context| {
                while !context.is_cancelled() {
                    thread::yield_now();
                }
                worker_exited.store(true, Ordering::Release);
            })
            .unwrap();
        runtime.cancel(handle).unwrap();

        for _ in 0..100 {
            if exited.load(Ordering::Acquire) {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        runtime.stop().unwrap();
        assert!(exited.load(Ordering::Acquire));
    }

    #[test]
    fn concurrent_spawn_cannot_escape_shutdown() {
        for _ in 0..100 {
            let runtime = Arc::new(Runtime::new());
            let barrier = Arc::new(Barrier::new(2));
            let exited = Arc::new(AtomicBool::new(false));
            let spawn_runtime = Arc::clone(&runtime);
            let spawn_barrier = Arc::clone(&barrier);
            let worker_exited = Arc::clone(&exited);
            let spawning = thread::spawn(move || {
                spawn_barrier.wait();
                spawn_runtime.spawn("racing-worker", move |context| {
                    while !context.is_cancelled() {
                        thread::yield_now();
                    }
                    worker_exited.store(true, Ordering::Release);
                })
            });

            barrier.wait();
            runtime.stop().unwrap();
            let spawn_result = spawning.join().unwrap();
            if spawn_result.is_ok() {
                assert!(exited.load(Ordering::Acquire));
            } else {
                assert_eq!(spawn_result, Err(RuntimeError::NotRunning));
            }
            assert_eq!(runtime.state(), RuntimeState::Stopped);
        }
    }
}
