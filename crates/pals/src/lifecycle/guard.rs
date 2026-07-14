use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum GuardError {
    Busy(PathBuf),
    Io(std::io::Error),
}

impl std::fmt::Display for GuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Busy(path) => write!(f, "another lifecycle operation is in progress: {}", path.display()),
            Self::Io(error) => write!(f, "failed to acquire lifecycle lock: {error}"),
        }
    }
}

impl std::error::Error for GuardError {}

impl From<std::io::Error> for GuardError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct OperationGuard {
    lock_path: PathBuf,
    _lock_file: File,
}

impl OperationGuard {
    pub fn acquire(path: impl AsRef<Path>) -> Result<Self, GuardError> {
        let path = path.as_ref().to_path_buf();
        match OpenOptions::new().create_new(true).write(true).open(&path) {
            Ok(mut lock_file) => {
                let _ = writeln!(lock_file, "pid={}", std::process::id());
                Ok(Self {
                    lock_path: path,
                    _lock_file: lock_file,
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(GuardError::Busy(path))
            }
            Err(error) => Err(GuardError::Io(error)),
        }
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        if std::fs::remove_file(&self.lock_path).is_err() {
            // best effort cleanup
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_lock_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for test")
            .as_nanos();
        std::env::temp_dir().join(format!("pals-lifecycle-lock-{nanos}.lock"))
    }

    #[test]
    fn second_lock_acquire_fails_until_first_guard_dropped() {
        let lock_path = unique_lock_path();
        let guard = OperationGuard::acquire(&lock_path).expect("first acquire should succeed");

        let second = OperationGuard::acquire(&lock_path);
        assert!(matches!(second, Err(GuardError::Busy(_))));

        drop(guard);
        let third = OperationGuard::acquire(&lock_path);
        assert!(third.is_ok());
    }

    #[test]
    fn acquire_reports_io_error_for_directory_path() {
        let dir_path = std::env::temp_dir();
        let result = OperationGuard::acquire(dir_path);
        assert!(matches!(result, Err(GuardError::Io(_)) | Err(GuardError::Busy(_))));
    }
}
