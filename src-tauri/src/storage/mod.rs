//! Local recent-jobs history (doc §3.3, AD-8).
//!
//! Stores job metadata and output locations only: never thumbnails, document
//! contents or copies of files. The schema is versioned with
//! `rusqlite_migration`; add new migrations to the end of `MIGRATIONS`, never
//! edit an existing one.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use docuflow_core::{CoreError, CoreResult};
use rusqlite::{Connection, params};
use rusqlite_migration::{M, Migrations};
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Number;

const MIGRATIONS: &[M<'static>] = &[M::up(
    "CREATE TABLE jobs (
        id            INTEGER PRIMARY KEY,
        operation     TEXT    NOT NULL,
        created_at    INTEGER NOT NULL DEFAULT (unixepoch()),
        status        TEXT    NOT NULL,
        source_names  TEXT    NOT NULL,  -- JSON array of file names
        output_paths  TEXT    NOT NULL,  -- JSON array of absolute paths
        input_bytes   INTEGER,
        output_bytes  INTEGER
    );",
)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    CompressImage,
    ResizeImage,
    ImagesToPdf,
    MergePdf,
    SplitPdf,
    OrganizePdf,
    CompressPdf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum JobStatus {
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewJob {
    pub operation: Operation,
    pub status: JobStatus,
    pub source_names: Vec<String>,
    pub output_paths: Vec<String>,
    pub input_bytes: Option<u64>,
    pub output_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HistoryOutput {
    pub path: String,
    /// `false` once the file has been moved or deleted.
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    #[specta(type = Number)]
    pub id: i64,
    pub operation: Operation,
    /// Unix timestamp in seconds.
    #[specta(type = Number)]
    pub created_at: i64,
    pub status: JobStatus,
    pub source_names: Vec<String>,
    pub outputs: Vec<HistoryOutput>,
    #[specta(type = Option<Number>)]
    pub input_bytes: Option<u64>,
    #[specta(type = Option<Number>)]
    pub output_bytes: Option<u64>,
}

pub struct History {
    conn: Mutex<Connection>,
}

impl History {
    pub fn open(path: &Path) -> CoreResult<Self> {
        Self::init(Connection::open(path).map_err(db_err)?)
    }

    pub fn open_in_memory() -> CoreResult<Self> {
        Self::init(Connection::open_in_memory().map_err(db_err)?)
    }

    fn init(mut conn: Connection) -> CoreResult<Self> {
        Migrations::from_slice(MIGRATIONS)
            .to_latest(&mut conn)
            .map_err(|err| {
                tracing::error!(%err, "history migration failed");
                CoreError::Internal
            })?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn record(&self, job: &NewJob) -> CoreResult<i64> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO jobs (operation, status, source_names, output_paths, input_bytes, output_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                to_text(&job.operation)?,
                to_text(&job.status)?,
                to_json(&job.source_names)?,
                to_json(&job.output_paths)?,
                job.input_bytes.map(to_i64),
                job.output_bytes.map(to_i64),
            ],
        )
        .map_err(db_err)?;
        Ok(conn.last_insert_rowid())
    }

    /// Most recent first.
    pub fn list(&self, limit: u32) -> CoreResult<Vec<HistoryEntry>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, operation, created_at, status, source_names, output_paths, input_bytes, output_bytes
                 FROM jobs ORDER BY created_at DESC, id DESC LIMIT ?1",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([limit], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                ))
            })
            .map_err(db_err)?;

        let mut entries = Vec::new();
        for row in rows {
            let (id, op, created_at, status, sources, outputs, input_bytes, output_bytes) =
                row.map_err(db_err)?;
            let output_paths: Vec<String> = from_json(&outputs)?;
            entries.push(HistoryEntry {
                id,
                operation: from_text(&op)?,
                created_at,
                status: from_text(&status)?,
                source_names: from_json(&sources)?,
                outputs: output_paths
                    .into_iter()
                    .map(|path| HistoryOutput {
                        available: Path::new(&path).is_file(),
                        path,
                    })
                    .collect(),
                input_bytes: input_bytes.map(|b| b.max(0) as u64),
                output_bytes: output_bytes.map(|b| b.max(0) as u64),
            });
        }
        Ok(entries)
    }

    /// Deletes job records only, never documents. `VACUUM` makes sure the
    /// deleted rows don't linger inside the database file.
    pub fn clear(&self) -> CoreResult<()> {
        let conn = self.conn();
        conn.execute_batch("DELETE FROM jobs; VACUUM;")
            .map_err(db_err)
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }
}

fn db_err(err: rusqlite::Error) -> CoreError {
    tracing::error!(%err, "history database error");
    CoreError::Internal
}

fn to_i64(bytes: u64) -> i64 {
    i64::try_from(bytes).unwrap_or(i64::MAX)
}

fn to_json<T: Serialize>(value: &T) -> CoreResult<String> {
    serde_json::to_string(value).map_err(|_| CoreError::Internal)
}

fn from_json<T: for<'de> Deserialize<'de>>(text: &str) -> CoreResult<T> {
    serde_json::from_str(text).map_err(|_| CoreError::Internal)
}

/// Enums are stored as their plain serde name, e.g. `compressImage`.
fn to_text<T: Serialize>(value: &T) -> CoreResult<String> {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(s)) => Ok(s),
        _ => Err(CoreError::Internal),
    }
}

fn from_text<T: for<'de> Deserialize<'de>>(text: &str) -> CoreResult<T> {
    serde_json::from_value(serde_json::Value::String(text.to_owned()))
        .map_err(|_| CoreError::Internal)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(output: &str) -> NewJob {
        NewJob {
            operation: Operation::MergePdf,
            status: JobStatus::Succeeded,
            source_names: vec!["a.pdf".into(), "b.pdf".into()],
            output_paths: vec![output.into()],
            input_bytes: Some(2_000),
            output_bytes: Some(1_500),
        }
    }

    #[test]
    fn migrations_are_valid() {
        assert!(Migrations::from_slice(MIGRATIONS).validate().is_ok());
    }

    #[test]
    fn records_lists_and_clears() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("merged.pdf");
        std::fs::write(&existing, b"%PDF").unwrap();

        let history = History::open_in_memory().unwrap();
        history.record(&job(existing.to_str().unwrap())).unwrap();
        history.record(&job("/moved/away.pdf")).unwrap();

        let entries = history.list(10).unwrap();
        assert_eq!(entries.len(), 2);
        // Most recent first.
        assert_eq!(entries[0].outputs[0].path, "/moved/away.pdf");
        assert!(!entries[0].outputs[0].available);
        assert!(entries[1].outputs[0].available);
        assert_eq!(entries[1].operation, Operation::MergePdf);
        assert_eq!(entries[1].source_names, vec!["a.pdf", "b.pdf"]);

        history.clear().unwrap();
        assert!(history.list(10).unwrap().is_empty());
        assert!(
            existing.exists(),
            "clearing history never deletes documents"
        );
    }

    #[test]
    fn enums_are_stored_as_plain_names() {
        assert_eq!(to_text(&Operation::OrganizePdf).unwrap(), "organizePdf");
        assert_eq!(
            from_text::<JobStatus>("cancelled").unwrap(),
            JobStatus::Cancelled
        );
    }
}
