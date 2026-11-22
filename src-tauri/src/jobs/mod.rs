//! Job queue (AD-4).
//!
//! Every operation runs through [`JobManager::run`]:
//! 1. it gets an id, announced on the channel as `queued`
//! 2. it waits for the single heavy-job slot (FIFO; one heavy job at a time)
//! 3. the work runs on a blocking thread, streaming `progress` events
//! 4. the command resolves with the work's result
//!
//! `cancel_job(id)` flips the job's cancel token; `docuflow-core` checks it
//! between steps and returns `CoreError::Cancelled`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard};

use docuflow_core::{CancelToken, CoreError, CoreResult, JobContext, Progress};
use serde::Serialize;
use specta::Type;
use tauri::async_runtime;
use tauri::ipc::Channel;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(
    tag = "event",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum JobEvent {
    Queued { job_id: u32 },
    Started,
    Progress(Progress),
}

#[derive(Debug, Default)]
pub struct JobManager {
    next_id: AtomicU32,
    active: Mutex<HashMap<u32, CancelToken>>,
    /// tokio's Mutex is fair, so queued jobs start in FIFO order.
    heavy_slot: async_runtime::Mutex<()>,
}

impl JobManager {
    pub async fn run<T, F>(&self, events: Channel<JobEvent>, work: F) -> CoreResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&JobContext) -> CoreResult<T> + Send + 'static,
    {
        let job_id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let cancel = CancelToken::new();
        self.active().insert(job_id, cancel.clone());
        let _ = events.send(JobEvent::Queued { job_id });

        let result = self.execute(events, cancel, work).await;

        self.active().remove(&job_id);
        if let Err(err) = &result {
            tracing::info!(job_id, code = err.code(), "job finished with error");
        }
        result
    }

    async fn execute<T, F>(
        &self,
        events: Channel<JobEvent>,
        cancel: CancelToken,
        work: F,
    ) -> CoreResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&JobContext) -> CoreResult<T> + Send + 'static,
    {
        let _slot = self.heavy_slot.lock().await;
        if cancel.is_cancelled() {
            return Err(CoreError::Cancelled);
        }
        let _ = events.send(JobEvent::Started);

        async_runtime::spawn_blocking(move || {
            let sink = move |p: Progress| {
                let _ = events.send(JobEvent::Progress(p));
            };
            let ctx = JobContext::new(cancel, &sink);
            work(&ctx)
        })
        .await
        .map_err(|_| CoreError::Internal)?
    }

    /// Returns `false` if the job already finished or never existed.
    pub fn cancel(&self, job_id: u32) -> bool {
        match self.active().get(&job_id) {
            Some(token) => {
                token.cancel();
                true
            }
            None => false,
        }
    }

    fn active(&self) -> MutexGuard<'_, HashMap<u32, CancelToken>> {
        self.active.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use docuflow_core::Stage;
    use std::sync::Arc;
    use tauri::ipc::InvokeResponseBody;

    fn recording_channel() -> (Channel<JobEvent>, Arc<Mutex<Vec<String>>>) {
        let log = Arc::new(Mutex::new(Vec::new()));
        let sink = log.clone();
        let channel = Channel::new(move |body| {
            if let InvokeResponseBody::Json(json) = body {
                sink.lock().unwrap().push(json);
            }
            Ok(())
        });
        (channel, log)
    }

    #[test]
    fn runs_work_and_streams_events() {
        let jobs = JobManager::default();
        let (channel, log) = recording_channel();

        let result = async_runtime::block_on(jobs.run(channel, |ctx| {
            ctx.report(Stage::Processing, 50.0);
            Ok(42)
        }));

        assert_eq!(result, Ok(42));
        let log = log.lock().unwrap();
        assert_eq!(log[0], r#"{"event":"queued","data":{"jobId":1}}"#);
        assert_eq!(log[1], r#"{"event":"started"}"#);
        assert!(log[2].contains(r#""event":"progress""#));
        assert!(jobs.active().is_empty());
    }

    #[test]
    fn cancelled_jobs_return_cancelled() {
        let jobs = Arc::new(JobManager::default());
        let (channel, _) = recording_channel();
        let jobs2 = jobs.clone();

        let result = async_runtime::block_on(jobs.run(channel, move |ctx| {
            assert!(jobs2.cancel(1));
            ctx.check_cancelled()?;
            Ok(())
        }));

        assert_eq!(result, Err(CoreError::Cancelled));
        assert!(!jobs.cancel(1), "finished jobs can't be cancelled");
    }
}
