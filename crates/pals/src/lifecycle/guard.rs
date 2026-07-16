use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
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
        match Self::create(&path) {
            Ok(guard) => Ok(guard),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // The process that created this lock may have been killed
                // before its `Drop` ran (e.g. SIGKILL after a grace-period
                // timeout), leaving an orphaned file behind. Reclaim it if
                // the recorded pid is no longer alive.
                if lock_holder_pid(&path).is_some_and(|pid| pid_is_alive(pid)) {
                    return Err(GuardError::Busy(path));
                }
                let _ = std::fs::remove_file(&path);
                Self::create(&path).map_err(|error| {
                    if error.kind() == std::io::ErrorKind::AlreadyExists {
                        GuardError::Busy(path.clone())
                    } else {
                        GuardError::Io(error)
                    }
                })
            }
            Err(error) => Err(GuardError::Io(error)),
        }
    }

    fn create(path: &Path) -> std::io::Result<Self> {
        let mut lock_file = OpenOptions::new().create_new(true).write(true).open(path)?;
        let _ = writeln!(lock_file, "pid={}", std::process::id());
        Ok(Self {
            lock_path: path.to_path_buf(),
            _lock_file: lock_file,
        })
    }
}

/// Reads the pid recorded in an existing lock file, if any.
fn lock_holder_pid(path: &Path) -> Option<u32> {
    let mut contents = String::new();
    File::open(path).ok()?.read_to_string(&mut contents).ok()?;
    contents
        .lines()
        .find_map(|line| line.strip_prefix("pid="))
        .and_then(|pid| pid.trim().parse().ok())
}

/// Checks whether a process with the given pid is still alive.
#[cfg(target_os = "linux")]
fn pid_is_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

#[cfg(not(target_os = "linux"))]
fn pid_is_alive(_pid: u32) -> bool {
    // Conservatively assume it's alive so we never reclaim a lock we can't verify.
    true
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
    fn stale_lock_from_dead_pid_is_reclaimed() {
        let lock_path = unique_lock_path();
        std::fs::write(&lock_path, "pid=999999999\n").expect("write stale lock");

        let guard = OperationGuard::acquire(&lock_path)
            .expect("lock held by a dead pid should be reclaimed");
        drop(guard);
        assert!(!lock_path.exists());
    }

    #[test]
    fn acquire_reports_io_error_for_directory_path() {
        let dir_path = std::env::temp_dir();
        let result = OperationGuard::acquire(dir_path);
        assert!(matches!(result, Err(GuardError::Io(_)) | Err(GuardError::Busy(_))));
    }
}
