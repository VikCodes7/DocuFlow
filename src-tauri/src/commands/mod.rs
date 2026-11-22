//! IPC commands: thin wrappers that check paths, then call `docuflow-core`
//! through the job queue. Every command is listed in `crate::specta_builder`
//! so its TypeScript binding is generated (AD-2).

use std::path::{Path, PathBuf};

use docuflow_core::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Number;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::access::AllowedPaths;
use crate::jobs::JobManager;
use crate::storage::{History, HistoryEntry};

const HISTORY_LIMIT: u32 = 200;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
}

/// Which kind of file a picker should accept.
#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum InputKind {
    Image,
    Pdf,
}

impl InputKind {
    fn filter(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Image => ("Images", &["jpg", "jpeg", "png", "webp"]),
            Self::Pdf => ("PDF documents", &["pdf"]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SelectedFile {
    /// Canonical absolute path. Pass it back to commands unchanged.
    pub path: String,
    pub name: String,
    #[specta(type = Number)]
    pub size_bytes: u64,
}

/// Sent when files are dropped on the window. They are already registered in
/// the allowed-path registry; the frontend decides which ones a tool accepts.
#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct FilesDropped {
    pub files: Vec<SelectedFile>,
}

/// Registers user-chosen files and describes them for the UI. Paths that
/// aren't regular files (e.g. dropped folders) are skipped.
pub fn register_files(
    access: &AllowedPaths,
    paths: impl IntoIterator<Item = PathBuf>,
) -> Vec<SelectedFile> {
    paths
        .into_iter()
        .filter_map(|p| access.allow_file(&p).ok())
        .filter_map(|p| describe(&p).ok())
        .collect()
}

fn describe(path: &Path) -> CoreResult<SelectedFile> {
    Ok(SelectedFile {
        path: path.to_string_lossy().into_owned(),
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        size_bytes: std::fs::metadata(path)?.len(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn app_info(app: AppHandle) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
    }
}

/// Opens a native file picker. Picked files are added to the allowed-path
/// registry, which is why the frontend has no dialog or fs permissions (AD-6).
#[tauri::command]
#[specta::specta]
pub async fn pick_input_files(
    app: AppHandle,
    kind: InputKind,
    multiple: bool,
) -> CoreResult<Vec<SelectedFile>> {
    let (filter_name, extensions) = kind.filter();
    let dialog = app.dialog().file().add_filter(filter_name, extensions);

    let picked = tauri::async_runtime::spawn_blocking(move || {
        if multiple {
            dialog.blocking_pick_files()
        } else {
            dialog.blocking_pick_file().map(|f| vec![f])
        }
    })
    .await
    .map_err(|_| CoreError::Internal)?
    .unwrap_or_default();

    let paths = picked.into_iter().filter_map(|f| f.into_path().ok());
    Ok(register_files(&app.state::<AllowedPaths>(), paths))
}

/// Opens a native folder picker for the output destination.
#[tauri::command]
#[specta::specta]
pub async fn pick_output_folder(app: AppHandle) -> CoreResult<Option<String>> {
    let dialog = app.dialog().file();
    let picked = tauri::async_runtime::spawn_blocking(move || dialog.blocking_pick_folder())
        .await
        .map_err(|_| CoreError::Internal)?;

    let Some(path) = picked.and_then(|f| f.into_path().ok()) else {
        return Ok(None);
    };
    let canonical = app.state::<AllowedPaths>().allow_dir(&path)?;
    Ok(Some(canonical.to_string_lossy().into_owned()))
}

/// Returns `false` if the job had already finished.
#[tauri::command]
#[specta::specta]
pub fn cancel_job(jobs: State<'_, JobManager>, job_id: u32) -> bool {
    jobs.cancel(job_id)
}

#[tauri::command]
#[specta::specta]
pub fn list_history(history: State<'_, History>) -> CoreResult<Vec<HistoryEntry>> {
    history.list(HISTORY_LIMIT)
}

/// Removes job records only. Never touches documents.
#[tauri::command]
#[specta::specta]
pub fn clear_history(history: State<'_, History>) -> CoreResult<()> {
    history.clear()
}

/// Opens the local log folder in Finder / Explorer, for support requests.
#[tauri::command]
#[specta::specta]
pub fn open_log_folder(app: AppHandle) -> CoreResult<()> {
    let dir = app.path().app_log_dir().map_err(|_| CoreError::Internal)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|_| CoreError::Internal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_files_skips_folders_and_missing_paths() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("scan.pdf");
        std::fs::write(&file, b"%PDF-1.7").unwrap();

        let access = AllowedPaths::default();
        let files = register_files(
            &access,
            [
                file.clone(),
                dir.path().to_path_buf(),
                dir.path().join("missing.pdf"),
            ],
        );

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "scan.pdf");
        assert_eq!(files[0].size_bytes, 8);
        assert!(access.check_input(&file).is_ok());
    }
}
