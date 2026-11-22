//! PDF page rendering via `pdfium-render`.
//!
//! PDFium is not thread-safe: the app layer must call into this module from a
//! single dedicated worker thread. Render only visible pages, at display
//! resolution, into the app's preview folder (served via the asset protocol).
//!
//! The PDFium library is loaded dynamically at runtime from
//! `src-tauri/resources/pdfium/`.
