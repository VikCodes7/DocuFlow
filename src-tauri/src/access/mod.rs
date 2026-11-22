//! Allowed-path registry (AD-6).
//!
//! Tauri capabilities don't restrict the app's own Rust code, so this registry
//! is what limits DocuFlow to files the user actually chose: files picked in a
//! dialog or dropped on the window, and folders picked as output destinations.
//! Every command must pass its paths through `check_input` / `check_output_dir`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use docuflow_core::{CoreError, CoreResult};

#[derive(Debug, Default)]
pub struct AllowedPaths {
    inner: Mutex<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    files: HashSet<PathBuf>,
    dirs: HashSet<PathBuf>,
}

impl AllowedPaths {
    /// Registers a user-selected file. Returns its canonical path.
    pub fn allow_file(&self, path: &Path) -> CoreResult<PathBuf> {
        let canonical = canonicalize(path)?;
        if !canonical.is_file() {
            return Err(CoreError::invalid_input("not a regular file"));
        }
        self.lock().files.insert(canonical.clone());
        Ok(canonical)
    }

    /// Registers a user-selected output folder. Returns its canonical path.
    pub fn allow_dir(&self, path: &Path) -> CoreResult<PathBuf> {
        let canonical = canonicalize(path)?;
        if !canonical.is_dir() {
            return Err(CoreError::invalid_input("not a folder"));
        }
        self.lock().dirs.insert(canonical.clone());
        Ok(canonical)
    }

    /// Returns the canonical path if the file was selected by the user.
    pub fn check_input(&self, path: &Path) -> CoreResult<PathBuf> {
        let canonical = canonicalize(path).map_err(|_| CoreError::PathNotAllowed)?;
        if self.lock().files.contains(&canonical) {
            Ok(canonical)
        } else {
            Err(CoreError::PathNotAllowed)
        }
    }

    /// Returns the canonical path if the folder was selected by the user.
    pub fn check_output_dir(&self, path: &Path) -> CoreResult<PathBuf> {
        let canonical = canonicalize(path).map_err(|_| CoreError::PathNotAllowed)?;
        if self.lock().dirs.contains(&canonical) {
            Ok(canonical)
        } else {
            Err(CoreError::PathNotAllowed)
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        // A poisoned lock only means another thread panicked mid-insert;
        // the sets are still valid.
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// `dunce` avoids `\\?\` prefixes on Windows so paths stay displayable.
fn canonicalize(path: &Path) -> CoreResult<PathBuf> {
    Ok(dunce::canonicalize(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_registered_files_are_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let picked = dir.path().join("picked.pdf");
        let other = dir.path().join("other.pdf");
        std::fs::write(&picked, b"x").unwrap();
        std::fs::write(&other, b"x").unwrap();

        let reg = AllowedPaths::default();
        reg.allow_file(&picked).unwrap();

        assert!(reg.check_input(&picked).is_ok());
        assert_eq!(reg.check_input(&other), Err(CoreError::PathNotAllowed));
    }

    #[test]
    fn path_tricks_resolve_to_the_same_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        let picked = dir.path().join("picked.pdf");
        std::fs::write(&picked, b"x").unwrap();

        let reg = AllowedPaths::default();
        reg.allow_file(&picked).unwrap();

        let roundabout = dir.path().join("sub").join("..").join("picked.pdf");
        assert!(reg.check_input(&roundabout).is_ok());
    }

    #[test]
    fn files_and_dirs_are_separate() {
        let dir = tempfile::tempdir().unwrap();
        let reg = AllowedPaths::default();
        reg.allow_dir(dir.path()).unwrap();

        assert!(reg.check_output_dir(dir.path()).is_ok());
        assert_eq!(reg.check_input(dir.path()), Err(CoreError::PathNotAllowed));
        assert!(reg.allow_file(dir.path()).is_err());
    }

    #[test]
    fn missing_paths_are_not_allowed() {
        let reg = AllowedPaths::default();
        let missing = Path::new("/definitely/not/here.pdf");
        assert_eq!(reg.check_input(missing), Err(CoreError::PathNotAllowed));
    }
}
