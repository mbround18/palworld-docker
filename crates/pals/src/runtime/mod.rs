pub mod docker;

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerStatus {
    pub exists: bool,
    pub running: bool,
    pub health: HealthStatus,
}

impl ContainerStatus {
    pub fn not_found() -> Self {
        Self {
            exists: false,
            running: false,
            health: HealthStatus::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContainerSpec {
    pub name: String,
    pub image: String,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub env: Vec<String>,
}

#[derive(Debug)]
pub enum RuntimeError {
    Io(std::io::Error),
    CommandFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
    Parse(String),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O failure: {err}"),
            Self::CommandFailed {
                command,
                code,
                stderr,
            } => write!(
                f,
                "command failed: {command} (code: {:?}) stderr: {}",
                code, stderr
            ),
            Self::Parse(message) => write!(f, "parse failure: {message}"),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<std::io::Error> for RuntimeError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub trait ContainerRuntime {
    fn ensure_image(&self, image: &str) -> Result<(), RuntimeError>;
    fn inspect(&self, name: &str) -> Result<ContainerStatus, RuntimeError>;
    fn create(&self, spec: &ContainerSpec) -> Result<(), RuntimeError>;
    fn start(&self, name: &str) -> Result<(), RuntimeError>;
    fn stop(&self, name: &str, timeout_seconds: u32) -> Result<(), RuntimeError>;
    fn remove(&self, name: &str, force: bool) -> Result<(), RuntimeError>;
    fn logs(&self, name: &str, follow: bool, tail: u32) -> Result<String, RuntimeError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_status_marks_container_as_missing() {
        let status = ContainerStatus::not_found();
        assert!(!status.exists);
        assert!(!status.running);
        assert_eq!(status.health, HealthStatus::None);
    }

    #[test]
    fn command_failed_error_displays_command_and_code() {
        let err = RuntimeError::CommandFailed {
            command: "docker inspect palworld".to_owned(),
            code: Some(1),
            stderr: "No such object".to_owned(),
        };
        let message = err.to_string();
        assert!(message.contains("docker inspect palworld"));
        assert!(message.contains("Some(1)"));
    }
}
