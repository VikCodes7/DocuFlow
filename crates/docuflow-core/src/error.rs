//! One error model with stable codes (AD-3).
//!
//! Every variant serializes as `{ "code": "...", "details": ... }`. The `code`
//! values are a public contract with the frontend, which maps each one to a
//! translated message: never rename a variant, only add new ones.
//!
//! Details must never contain document contents, filenames or paths.

use serde::Serialize;
use specta::Type;
// JS numbers are exact up to 2^53, plenty for byte and pixel counts.
use specta_typescript::Number;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Clone, PartialEq, thiserror::Error, Serialize, Type)]
#[serde(tag = "code", content = "details", rename_all_fields = "camelCase")]
pub enum CoreError {
    #[error("the PDF is password-protected")]
    PdfEncrypted,

    #[error("the PDF is damaged or not a valid PDF")]
    PdfMalformed,

    #[error("the target size cannot be reached above the quality floor")]
    TargetUnreachable {
        /// Smallest size achievable at the quality floor, in bytes.
        #[specta(type = Number)]
        smallest_achievable_bytes: u64,
    },

    #[error("the file is larger than the {limit_bytes}-byte limit")]
    FileTooLarge {
        #[specta(type = Number)]
        limit_bytes: u64,
        #[specta(type = Number)]
        actual_bytes: u64,
    },

    #[error("the image exceeds the {limit_pixels}-pixel limit")]
    ImageTooLarge {
        #[specta(type = Number)]
        limit_pixels: u64,
        #[specta(type = Number)]
        actual_pixels: u64,
    },

    #[error("the PDF exceeds the {limit_pages}-page limit")]
    TooManyPages { limit_pages: u32, actual_pages: u32 },

    #[error("this file format is not supported")]
    UnsupportedFormat,

    #[error("the file or folder was not selected by the user")]
    PathNotAllowed,

    #[error("the operation was cancelled")]
    Cancelled,

    #[error("invalid input: {reason}")]
    InvalidInput {
        /// Short, content-free explanation, e.g. "duplicate page index".
        reason: String,
    },

    #[error("file system error ({kind})")]
    Io {
        /// `std::io::ErrorKind` name only, never the path.
        kind: String,
    },

    #[error("unexpected internal error")]
    Internal,
}

impl CoreError {
    /// Stable code, identical to the serialized `code` field.
    pub fn code(&self) -> &'static str {
        match self {
            Self::PdfEncrypted => "PdfEncrypted",
            Self::PdfMalformed => "PdfMalformed",
            Self::TargetUnreachable { .. } => "TargetUnreachable",
            Self::FileTooLarge { .. } => "FileTooLarge",
            Self::ImageTooLarge { .. } => "ImageTooLarge",
            Self::TooManyPages { .. } => "TooManyPages",
            Self::UnsupportedFormat => "UnsupportedFormat",
            Self::PathNotAllowed => "PathNotAllowed",
            Self::Cancelled => "Cancelled",
            Self::InvalidInput { .. } => "InvalidInput",
            Self::Io { .. } => "Io",
            Self::Internal => "Internal",
        }
    }

    pub fn invalid_input(reason: impl Into<String>) -> Self {
        Self::InvalidInput {
            reason: reason.into(),
        }
    }
}

impl From<std::io::Error> for CoreError {
    fn from(err: std::io::Error) -> Self {
        // Deliberately drops the message: io errors can embed paths.
        Self::Io {
            kind: format!("{:?}", err.kind()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_with_stable_code_and_details() {
        let err = CoreError::FileTooLarge {
            limit_bytes: 10,
            actual_bytes: 20,
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "FileTooLarge");
        assert_eq!(json["details"]["limitBytes"], 10);
        assert_eq!(err.code(), "FileTooLarge");
    }

    #[test]
    fn unit_variants_serialize_code_only() {
        let json = serde_json::to_value(CoreError::Cancelled).unwrap();
        assert_eq!(json, serde_json::json!({ "code": "Cancelled" }));
    }

    #[test]
    fn io_errors_do_not_leak_paths() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "/Users/someone/secret.pdf");
        let err = CoreError::from(io);
        let json = serde_json::to_string(&err).unwrap();
        assert!(!json.contains("secret"));
        assert_eq!(err.code(), "Io");
    }
}
