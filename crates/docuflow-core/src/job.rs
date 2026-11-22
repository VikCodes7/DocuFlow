//! Job primitives shared by every operation (AD-4).
//!
//! The app layer owns the queue and the progress channel; processing code only
//! sees a [`JobContext`], reports progress through it and calls
//! [`JobContext::check_cancelled`] between steps.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use specta::Type;

use crate::error::{CoreError, CoreResult};

/// Cheap, cloneable cancellation flag.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Coarse phase of a job, shown in the UI next to the progress bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Reading,
    Processing,
    Encoding,
    Writing,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// 0.0 to 100.0
    pub percent: f32,
    pub stage: Stage,
}

/// Handed to every processing function.
pub struct JobContext<'a> {
    cancel: CancelToken,
    on_progress: &'a (dyn Fn(Progress) + Send + Sync),
}

impl<'a> JobContext<'a> {
    pub fn new(cancel: CancelToken, on_progress: &'a (dyn Fn(Progress) + Send + Sync)) -> Self {
        Self {
            cancel,
            on_progress,
        }
    }

    /// Returns `Err(CoreError::Cancelled)` once the user has cancelled the job.
    pub fn check_cancelled(&self) -> CoreResult<()> {
        if self.cancel.is_cancelled() {
            Err(CoreError::Cancelled)
        } else {
            Ok(())
        }
    }

    pub fn report(&self, stage: Stage, percent: f32) {
        (self.on_progress)(Progress {
            percent: percent.clamp(0.0, 100.0),
            stage,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn cancellation_is_observed() {
        let token = CancelToken::new();
        let noop = |_: Progress| {};
        let ctx = JobContext::new(token.clone(), &noop);
        assert!(ctx.check_cancelled().is_ok());
        token.cancel();
        assert_eq!(ctx.check_cancelled(), Err(CoreError::Cancelled));
    }

    #[test]
    fn progress_is_clamped_and_forwarded() {
        let seen = Mutex::new(Vec::new());
        let sink = |p: Progress| seen.lock().unwrap().push(p.percent);
        let ctx = JobContext::new(CancelToken::new(), &sink);
        ctx.report(Stage::Processing, 150.0);
        ctx.report(Stage::Processing, -1.0);
        assert_eq!(*seen.lock().unwrap(), vec![100.0, 0.0]);
    }
}
