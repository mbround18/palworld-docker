use super::{ContainerRuntime, ContainerSpec, ContainerStatus, HealthStatus, RuntimeError};
use std::process::Command;

pub struct DockerCliRuntime;

impl DockerCliRuntime {
    fn is_no_such_object(stderr: &str) -> bool {
        stderr.to_ascii_lowercase().contains("no such object")
    }

    fn inspect_args(name: &str) -> Vec<String> {
        vec![
            "inspect".to_owned(),
            "--format".to_owned(),
            "{{.State.Running}} {{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}"
                .to_owned(),
            name.to_owned(),
        ]
    }

    fn create_args(spec: &ContainerSpec) -> Vec<String> {
        let mut args = vec!["create".to_owned(), "--name".to_owned(), spec.name.clone()];
        for port in &spec.ports {
            args.push("-p".to_owned());
            args.push(port.clone());
        }
        for volume in &spec.volumes {
            args.push("-v".to_owned());
            args.push(volume.clone());
        }
        for env_var in &spec.env {
            args.push("-e".to_owned());
            args.push(env_var.clone());
        }
        args.push(spec.image.clone());
        args
    }

    fn logs_args(name: &str, follow: bool, tail: u32) -> Vec<String> {
        let mut args = vec!["logs".to_owned()];
        if follow {
            args.push("--follow".to_owned());
        }
        args.push("--tail".to_owned());
        args.push(tail.to_string());
        args.push(name.to_owned());
        args
    }

    fn parse_inspect(stdout: &str) -> Result<ContainerStatus, RuntimeError> {
        let mut parts = stdout.split_whitespace();
        let running = parts
            .next()
            .ok_or_else(|| RuntimeError::Parse("missing running state".to_owned()))?
            == "true";
        let health = match parts.next().unwrap_or("none") {
            "healthy" => HealthStatus::Healthy,
            "unhealthy" => HealthStatus::Unhealthy,
            "none" => HealthStatus::None,
            _ => HealthStatus::Unknown,
        };

        Ok(ContainerStatus {
            exists: true,
            running,
            health,
        })
    }

    fn docker_output(args: &[String]) -> Result<std::process::Output, RuntimeError> {
        let output = Command::new("docker").args(args).output()?;
        Ok(output)
    }

    fn command_failed(args: &[String], output: &std::process::Output) -> RuntimeError {
        RuntimeError::CommandFailed {
            command: format!("docker {}", args.join(" ")),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        }
    }
}

impl ContainerRuntime for DockerCliRuntime {
    fn ensure_image(&self, image: &str) -> Result<(), RuntimeError> {
        let inspect_args = vec![
            "image".to_owned(),
            "inspect".to_owned(),
            image.to_owned(),
        ];
        let inspect = Self::docker_output(&inspect_args)?;
        if inspect.status.success() {
            return Ok(());
        }

        let pull_args = vec!["pull".to_owned(), image.to_owned()];
        let pull = Self::docker_output(&pull_args)?;
        if pull.status.success() {
            Ok(())
        } else {
            Err(Self::command_failed(&pull_args, &pull))
        }
    }

    fn inspect(&self, name: &str) -> Result<ContainerStatus, RuntimeError> {
        let args = Self::inspect_args(name);
        let output = Self::docker_output(&args)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if Self::is_no_such_object(&stderr) {
                return Ok(ContainerStatus::not_found());
            }
            return Err(Self::command_failed(&args, &output));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_inspect(&stdout)
    }

    fn create(&self, spec: &ContainerSpec) -> Result<(), RuntimeError> {
        let args = Self::create_args(spec);

        let output = Self::docker_output(&args)?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Self::command_failed(&args, &output))
        }
    }

    fn start(&self, name: &str) -> Result<(), RuntimeError> {
        let args = vec!["start".to_owned(), name.to_owned()];
        let output = Self::docker_output(&args)?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Self::command_failed(&args, &output))
        }
    }

    fn stop(&self, name: &str, timeout_seconds: u32) -> Result<(), RuntimeError> {
        let args = vec![
            "stop".to_owned(),
            "--time".to_owned(),
            timeout_seconds.to_string(),
            name.to_owned(),
        ];
        let output = Self::docker_output(&args)?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Self::command_failed(&args, &output))
        }
    }

    fn remove(&self, name: &str, force: bool) -> Result<(), RuntimeError> {
        let mut args = vec!["rm".to_owned()];
        if force {
            args.push("-f".to_owned());
        }
        args.push(name.to_owned());

        let output = Self::docker_output(&args)?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Self::command_failed(&args, &output))
        }
    }

    fn logs(&self, name: &str, follow: bool, tail: u32) -> Result<String, RuntimeError> {
        let args = Self::logs_args(name, follow, tail);

        let output = Self::docker_output(&args)?;
        if !output.status.success() {
            return Err(Self::command_failed(&args, &output));
        }

        let mut logs = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            if !logs.is_empty() {
                logs.push('\n');
            }
            logs.push_str(stderr.trim());
        }
        Ok(logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_inspect_handles_running_healthy() {
        let status = DockerCliRuntime::parse_inspect("true healthy").expect("parse should work");
        assert!(status.exists);
        assert!(status.running);
        assert_eq!(status.health, HealthStatus::Healthy);
    }

    #[test]
    fn parse_inspect_handles_stopped_with_no_healthcheck() {
        let status = DockerCliRuntime::parse_inspect("false none").expect("parse should work");
        assert!(status.exists);
        assert!(!status.running);
        assert_eq!(status.health, HealthStatus::None);
    }

    #[test]
    fn parse_inspect_requires_running_field() {
        let error = DockerCliRuntime::parse_inspect("").expect_err("parse should fail");
        assert!(matches!(error, RuntimeError::Parse(_)));
    }

    #[test]
    fn create_args_include_ports_volumes_and_env() {
        let spec = ContainerSpec {
            name: "palworld".to_owned(),
            image: "mbround18/palworld-docker:latest".to_owned(),
            ports: vec!["8211:8211/udp".to_owned()],
            volumes: vec!["./data:/home/steam/palworld".to_owned()],
            env: vec!["PRESET=Normal".to_owned()],
        };
        let args = DockerCliRuntime::create_args(&spec);
        assert_eq!(
            args,
            vec![
                "create".to_owned(),
                "--name".to_owned(),
                "palworld".to_owned(),
                "-p".to_owned(),
                "8211:8211/udp".to_owned(),
                "-v".to_owned(),
                "./data:/home/steam/palworld".to_owned(),
                "-e".to_owned(),
                "PRESET=Normal".to_owned(),
                "mbround18/palworld-docker:latest".to_owned()
            ]
        );
    }

    #[test]
    fn logs_args_include_follow_and_tail() {
        let args = DockerCliRuntime::logs_args("palworld", true, 50);
        assert_eq!(
            args,
            vec![
                "logs".to_owned(),
                "--follow".to_owned(),
                "--tail".to_owned(),
                "50".to_owned(),
                "palworld".to_owned()
            ]
        );
    }

    #[test]
    fn no_such_object_detection_is_case_insensitive() {
        assert!(DockerCliRuntime::is_no_such_object("Error: No such object: foo"));
        assert!(DockerCliRuntime::is_no_such_object("error: no such object: foo"));
    }
}
