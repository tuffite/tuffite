//! Shared lifecycle controls. Application traits return native Send futures.
use crate::supervisor::CancellationToken;
use crate::{ApplicationError, ApplicationResult};
use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::task::Poll;
use std::time::{Duration, Instant};

pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub(crate) const LIFECYCLE_TIMEOUT: Duration = Duration::from_secs(30);

/// Read-only, cloneable notification that ordinary application work must stop.
#[derive(Clone)]
pub struct StopSignal(pub(crate) CancellationToken);
impl std::fmt::Debug for StopSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StopSignal")
            .field("stopping", &self.is_stopping())
            .finish()
    }
}
impl StopSignal {
    #[must_use]
    pub fn is_stopping(&self) -> bool {
        self.0.is_cancelled()
    }
    /// Waits without polling. Dropping this future unregisters its waiter.
    pub async fn cancelled(&self) {
        self.0.cancelled().await;
    }
}

/// Why an initialized application instance is being released.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ShutdownReason {
    /// The host is retiring the instance or exiting normally.
    Requested,
    /// Browser initialization failed, timed out, or was interrupted.
    InitializationFailed,
}

/// Cleanup budget, separate from the already-fired ordinary-work stop signal.
#[derive(Clone, Debug)]
pub struct ShutdownContext {
    reason: ShutdownReason,
    deadline: Instant,
    cancellation: StopSignal,
}
impl ShutdownContext {
    pub(crate) fn new(reason: ShutdownReason, deadline: Instant) -> Self {
        Self {
            reason,
            deadline,
            cancellation: StopSignal(CancellationToken::new()),
        }
    }
    #[must_use]
    pub fn reason(&self) -> ShutdownReason {
        self.reason
    }
    #[must_use]
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
    #[must_use]
    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }
    /// Fired at the cleanup deadline; distinct from `Context::stop_signal()`.
    pub async fn cancelled(&self) {
        self.cancellation.cancelled().await;
    }
    pub(crate) fn run<T>(
        &self,
        future: impl Future<Output = ApplicationResult<T>>,
    ) -> ApplicationResult<T> {
        run_until(future, &self.cancellation.0, self.deadline)
    }
}

// One interruptible timer per lifecycle operation. It is joined when the operation
// completes; ordinary requests and stop-signal waiters do not create timer threads.
struct Deadline {
    done: Arc<(Mutex<bool>, Condvar)>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Deadline {
    fn new(deadline: Instant, token: CancellationToken) -> Self {
        let done = Arc::new((Mutex::new(false), Condvar::new()));
        let worker = done.clone();
        let thread = std::thread::spawn(move || {
            let (lock, condition) = &*worker;
            let mut done = lock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            while !*done {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    drop(done);
                    token.cancel();
                    return;
                }
                done = condition
                    .wait_timeout(done, remaining)
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .0;
            }
        });
        Self {
            done,
            thread: Some(thread),
        }
    }
}
impl Drop for Deadline {
    fn drop(&mut self) {
        *self
            .done
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
        self.done.1.notify_all();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Drives a cooperative future on its caller's worker, never on Chromium UI.
/// A future that blocks inside poll still cannot be forcibly interrupted safely.
pub(crate) fn run_until<T>(
    future: impl Future<Output = ApplicationResult<T>>,
    stop: &CancellationToken,
    deadline: Instant,
) -> ApplicationResult<T> {
    let timeout = CancellationToken::new();
    let _timer = Deadline::new(deadline, timeout.clone());
    let mut future = std::pin::pin!(future);
    let mut stopped = std::pin::pin!(stop.cancelled());
    let mut expired = std::pin::pin!(timeout.cancelled());
    futures_executor::block_on(poll_fn(|cx| {
        if expired.as_mut().poll(cx).is_ready() {
            stop.cancel();
            // Give cooperative cleanup one poll to observe its deadline signal.
            let _ = future.as_mut().poll(cx);
            return Poll::Ready(Err(ApplicationError::new("lifecycle deadline exceeded")));
        }
        if stopped.as_mut().poll(cx).is_ready() {
            return Poll::Ready(Err(ApplicationError::new("lifecycle interrupted")));
        }
        future.as_mut().poll(cx)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn cleanup_deadline_wakes_its_own_signal_and_drops_pending_future() {
        let context = ShutdownContext::new(
            ShutdownReason::Requested,
            Instant::now() + Duration::from_millis(20),
        );
        let observed = AtomicBool::new(false);
        let result = context.run(async {
            context.cancelled().await;
            observed.store(true, Ordering::Release);
            Ok(())
        });
        assert!(result.unwrap_err().to_string().contains("deadline"));
        assert!(observed.load(Ordering::Acquire));
    }

    #[test]
    fn ordinary_stop_does_not_cancel_cleanup_budget() {
        let stop = StopSignal(CancellationToken::new());
        stop.0.cancel();
        let context = ShutdownContext::new(
            ShutdownReason::Requested,
            Instant::now() + Duration::from_secs(1),
        );
        assert!(stop.is_stopping());
        assert!(!context.cancellation.is_stopping());
        context.run(async { Ok(()) }).unwrap();
        assert!(!context.cancellation.is_stopping());
    }

    #[test]
    fn stop_wakes_pending_initialization_without_waiting_for_deadline() {
        let stop = CancellationToken::new();
        let stopping = stop.clone();
        let worker = std::thread::spawn(move || {
            run_until(
                std::future::pending::<ApplicationResult<()>>(),
                &stopping,
                Instant::now() + Duration::from_secs(5),
            )
        });
        stop.cancel();
        assert!(
            worker
                .join()
                .unwrap()
                .unwrap_err()
                .to_string()
                .contains("interrupted")
        );
    }
}
