//! Image engine (doc §7).
//!
//! Planned:
//! - `resize`: exact pixels / percentage with `fast_image_resize` (Lanczos3).
//! - `compress_to_target`: binary search over encoder quality, encoding in
//!   memory, quality floor 40 for JPEG/WebP (§6.4, §7.2).
//! - `encode`: JPEG via `mozjpeg`, WebP via `webp`, PNG via `image` + `oxipng`.
//! - EXIF orientation via `kamadak-exif`; metadata is dropped on re-encode.
//!
//! Every entry point takes a `&JobContext` and checks `Limits` before decoding.
