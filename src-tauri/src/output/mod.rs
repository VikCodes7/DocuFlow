//! Safe output writing (AD-5).
//!
//! Output is written to a temp file in the destination folder, flushed to
//! disk, then moved into place without ever replacing an existing file. If the
//! name is taken, ` (1)`, ` (2)`, ... is appended. If writing fails or the job
//! is cancelled, the temp file is removed automatically.

use std::fs::File;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use docuflow_core::{CoreError, CoreResult};

const MAX_NAME_ATTEMPTS: u32 = 1_000;

/// Default output name, e.g. `("photo", "compressed", "jpg")` -> `photo-compressed.jpg`.
pub fn default_name(input_stem: &str, operation: &str, extension: &str) -> String {
    format!("{input_stem}-{operation}.{extension}")
}

/// `photo.jpg` with n=0 -> `photo.jpg`; n=2 -> `photo (2).jpg`.
fn numbered(file_name: &str, n: u32) -> String {
    if n == 0 {
        return file_name.to_owned();
    }
    match file_name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => format!("{stem} ({n}).{ext}"),
        _ => format!("{file_name} ({n})"),
    }
}

/// Writes a new file named `file_name` (or a numbered variant) inside `dir`.
///
/// `dir` must already have been checked with `AllowedPaths::check_output_dir`.
/// Returns the final path.
pub fn write_new_file<F>(dir: &Path, file_name: &str, write: F) -> CoreResult<PathBuf>
where
    F: FnOnce(&mut File) -> CoreResult<()>,
{
    if file_name.contains(['/', '\\']) || file_name.is_empty() {
        return Err(CoreError::invalid_input(
            "output name must be a plain file name",
        ));
    }

    let mut temp = tempfile::Builder::new()
        .prefix(".docuflow-")
        .suffix(".tmp")
        .tempfile_in(dir)?;

    // On error `temp` is dropped here, which deletes it.
    write(temp.as_file_mut())?;
    temp.as_file().sync_all()?;

    for n in 0..MAX_NAME_ATTEMPTS {
        let target = dir.join(numbered(file_name, n));
        match temp.persist_noclobber(&target) {
            Ok(_) => return Ok(target),
            Err(err) if err.error.kind() == ErrorKind::AlreadyExists => temp = err.file,
            Err(err) => return Err(err.error.into()),
        }
    }
    Err(CoreError::invalid_input("too many files with this name"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn builds_default_and_numbered_names() {
        assert_eq!(
            default_name("photo", "compressed", "jpg"),
            "photo-compressed.jpg"
        );
        assert_eq!(numbered("photo.jpg", 0), "photo.jpg");
        assert_eq!(numbered("photo.jpg", 2), "photo (2).jpg");
        assert_eq!(numbered("archive.tar.gz", 1), "archive.tar (1).gz");
        assert_eq!(numbered("README", 1), "README (1)");
        assert_eq!(numbered(".hidden", 1), ".hidden (1)");
    }

    #[test]
    fn writes_and_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("out.pdf"), b"original").unwrap();

        let path = write_new_file(dir.path(), "out.pdf", |f| {
            f.write_all(b"new")?;
            Ok(())
        })
        .unwrap();

        assert_eq!(path.file_name().unwrap(), "out (1).pdf");
        assert_eq!(
            std::fs::read(dir.path().join("out.pdf")).unwrap(),
            b"original"
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        assert_eq!(entries(dir.path()), vec!["out (1).pdf", "out.pdf"]);
    }

    #[test]
    fn failure_leaves_no_files_behind() {
        let dir = tempfile::tempdir().unwrap();
        let result = write_new_file(dir.path(), "out.pdf", |f| {
            f.write_all(b"partial")?;
            Err(CoreError::Cancelled)
        });

        assert_eq!(result, Err(CoreError::Cancelled));
        assert!(entries(dir.path()).is_empty());
    }

    #[test]
    fn rejects_names_with_separators() {
        let dir = tempfile::tempdir().unwrap();
        let result = write_new_file(dir.path(), "../escape.pdf", |_| Ok(()));
        assert_eq!(result.unwrap_err().code(), "InvalidInput");
    }
}
