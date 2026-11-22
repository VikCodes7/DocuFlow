//! Privacy-safe local logging (AD-9).
//!
//! Rules:
//! - Logs stay on the device, in the app's log folder.
//! - Never log document contents, file names or paths. Use [`file_ref`] to
//!   refer to a file: it returns a placeholder like `<file:3>` that is stable
//!   for the session but reveals nothing about the file.
//! - Log error *codes* (`CoreError::code()`), not raw error messages.
//!
//! Files rotate daily and the last 7 are kept. `tracing-appender` has no
//! size-based rotation, so the 5 MB cap from AD-9 is approximated by daily
//! rotation plus `info` as the default level.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::EnvFilter;

const KEEP_FILES: usize = 7;

/// Keeps the background log writer alive; dropping it flushes the log.
pub struct LogGuard(#[allow(dead_code)] WorkerGuard);

/// Starts file logging. Returns `None` (and logs nothing) if the log folder
/// can't be used, so a logging problem never prevents the app from starting.
pub fn init(log_dir: &Path) -> Option<LogGuard> {
    std::fs::create_dir_all(log_dir).ok()?;
    let appender = Builder::new()
        .rotation(Rotation::DAILY)
        .filename_prefix("docuflow")
        .filename_suffix("log")
        .max_log_files(KEEP_FILES)
        .build(log_dir)
        .ok()?;
    let (writer, guard) = tracing_appender::non_blocking(appender);

    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_env_filter(
            EnvFilter::try_from_env("DOCUFLOW_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .try_init()
        .ok()?;
    Some(LogGuard(guard))
}

static FILE_REFS: LazyLock<Mutex<HashMap<PathBuf, usize>>> = LazyLock::new(Default::default);

/// Anonymous, session-stable placeholder for a file, safe to put in logs.
pub fn file_ref(path: &Path) -> String {
    let mut refs = FILE_REFS.lock().unwrap_or_else(|e| e.into_inner());
    let next = refs.len() + 1;
    let n = *refs.entry(path.to_path_buf()).or_insert(next);
    format!("<file:{n}>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_refs_are_stable_and_reveal_nothing() {
        let a = file_ref(Path::new("/Users/someone/Taxes 2026.pdf"));
        let b = file_ref(Path::new("/Users/someone/passport.jpg"));
        assert_eq!(a, file_ref(Path::new("/Users/someone/Taxes 2026.pdf")));
        assert_ne!(a, b);
        assert!(a.starts_with("<file:") && !a.contains("Taxes"));
    }
}
