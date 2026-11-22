//! PDF engine (doc §8), built on `lopdf`.
//!
//! Planned:
//! - `merge`, `split`
//! - `organize_pages`: reorder and remove in one call (§8.1), including
//!   reference cleanup, `prune_objects()` and a fresh (non-incremental) save
//! - `compress`: Low / Medium / High presets (§4.6)
//! - `images_to_pdf`: JPEG embedded as DCTDecode without re-encoding
//!
//! Encrypted PDFs return `CoreError::PdfEncrypted` unless they open with an
//! empty password (§6.4).
