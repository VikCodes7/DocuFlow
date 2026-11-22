//! Input limits, checked before any decoding (AD-7).
//!
//! Defaults are starting points; tune them during the proof of concept.

use crate::error::{CoreError, CoreResult};

const MB: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub max_image_bytes: u64,
    pub max_image_pixels: u64,
    pub max_pdf_bytes: u64,
    pub max_pdf_pages: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_image_bytes: 200 * MB,
            max_image_pixels: 100_000_000,
            max_pdf_bytes: 500 * MB,
            max_pdf_pages: 2_000,
        }
    }
}

impl Limits {
    pub fn check_image_file(&self, bytes: u64) -> CoreResult<()> {
        check_bytes(bytes, self.max_image_bytes)
    }

    pub fn check_image_dimensions(&self, width: u32, height: u32) -> CoreResult<()> {
        let pixels = u64::from(width) * u64::from(height);
        if pixels > self.max_image_pixels {
            return Err(CoreError::ImageTooLarge {
                limit_pixels: self.max_image_pixels,
                actual_pixels: pixels,
            });
        }
        Ok(())
    }

    pub fn check_pdf_file(&self, bytes: u64) -> CoreResult<()> {
        check_bytes(bytes, self.max_pdf_bytes)
    }

    pub fn check_pdf_pages(&self, pages: u32) -> CoreResult<()> {
        if pages > self.max_pdf_pages {
            return Err(CoreError::TooManyPages {
                limit_pages: self.max_pdf_pages,
                actual_pages: pages,
            });
        }
        Ok(())
    }
}

fn check_bytes(actual: u64, limit: u64) -> CoreResult<()> {
    if actual > limit {
        return Err(CoreError::FileTooLarge {
            limit_bytes: limit,
            actual_bytes: actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_values_at_the_limit() {
        let l = Limits::default();
        assert!(l.check_image_file(200 * MB).is_ok());
        assert!(l.check_image_dimensions(10_000, 10_000).is_ok());
        assert!(l.check_pdf_pages(2_000).is_ok());
    }

    #[test]
    fn rejects_values_over_the_limit() {
        let l = Limits::default();
        assert_eq!(
            l.check_image_file(200 * MB + 1).unwrap_err().code(),
            "FileTooLarge"
        );
        assert_eq!(
            l.check_image_dimensions(10_001, 10_000).unwrap_err().code(),
            "ImageTooLarge"
        );
        assert_eq!(
            l.check_pdf_file(500 * MB + 1).unwrap_err().code(),
            "FileTooLarge"
        );
        assert_eq!(l.check_pdf_pages(2_001).unwrap_err().code(), "TooManyPages");
    }

    #[test]
    fn huge_dimensions_do_not_overflow() {
        let l = Limits::default();
        assert!(l.check_image_dimensions(u32::MAX, u32::MAX).is_err());
    }
}
