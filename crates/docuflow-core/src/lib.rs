//! DocuFlow processing core.
//!
//! Everything that touches image or PDF data lives here, independent of Tauri
//! (AD-1), so it can be unit-tested, benchmarked and reused without the app.

pub mod error;
pub mod images;
pub mod job;
pub mod limits;
pub mod pdf;
pub mod render;

pub use error::{CoreError, CoreResult};
pub use job::{CancelToken, JobContext, Progress, Stage};
pub use limits::Limits;
