use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Command as StdCommand;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Duration;

fn unique_name(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    format!("{prefix}-{nanos}")
}

fn unique_lock() -> String {
    format!("/tmp/pals-lock-{}.lock", unique_name("test"))
}

struct TestContainer {
    name: String,
}

impl TestContainer {
    fn create(name: String) -> Self {
        let _ = StdCommand::new("docker").args(["rm", "-f", &name]).output();
        let output = StdCommand::new("docker")
            .args([
                "create",
                "--name",
                &name,
                "busybox:latest",
                "sh",
                "-c",
                "echo ready; sleep 120",
            ])
            .output()
            .expect("docker create should run");
        assert!(
            output.status.success(),
            "docker create should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Self { name }
    }

    fn start(&self) {
        let output = StdCommand::new("docker")
            .args(["start", &self.name])
            .output()
            .expect("docker start should run");
        assert!(
            output.status.success(),
            "docker start should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

impl Drop for TestContainer {
    fn drop(&mut self) {
        let _ = StdCommand::new("docker")
            .args(["rm", "-f", &self.name])
            .output();
    }
}

#[test]
fn status_json_reports_not_found_for_missing_container() {
    let name = unique_name("missing-status");
    let lock = unique_lock();
    let mut cmd = Command::cargo_bin("palworld").expect("binary should build");
    cmd.args(["--output", "json", "status", "--name", &name]);
    cmd.env("PALWORLD_LIFECYCLE_LOCK", lock);
    cmd.assert()
        .code(2)
        .stderr(predicate::str::contains("\"code\":\"not_found\""));
}

#[test]
fn logs_json_reports_runtime_failure_for_missing_container() {
    let name = unique_name("missing-logs");
    let lock = unique_lock();
    let mut cmd = Command::cargo_bin("palworld").expect("binary should build");
    cmd.args(["--output", "json", "logs", "--name", &name, "--tail", "5"]);
    cmd.env("PALWORLD_LIFECYCLE_LOCK", lock);
    cmd.assert()
        .code(1)
        .stderr(predicate::str::contains("\"code\":\"runtime_failure\""));
}

#[test]
fn remove_json_is_success_when_container_missing() {
    let name = unique_name("missing-remove");
    let lock = unique_lock();
    let mut cmd = Command::cargo_bin("palworld").expect("binary should build");
    cmd.args(["--output", "json", "remove", "--name", &name]);
    cmd.env("PALWORLD_LIFECYCLE_LOCK", lock);
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("\"ok\":true"));
}

#[test]
fn container_start_json_reports_runtime_failure_for_missing_container() {
    let name = unique_name("missing-start");
    let lock = unique_lock();
    let mut cmd = Command::cargo_bin("palworld").expect("binary should build");
    cmd.args(["--output", "json", "container-start", "--name", &name]);
    cmd.env("PALWORLD_LIFECYCLE_LOCK", lock);
    cmd.assert()
        .code(1)
        .stderr(predicate::str::contains("\"code\":\"runtime_failure\""));
}

#[test]
fn container_stop_json_is_success_for_missing_container() {
    let name = unique_name("missing-stop");
    let lock = unique_lock();
    let mut cmd = Command::cargo_bin("palworld").expect("binary should build");
    cmd.args(["--output", "json", "container-stop", "--name", &name]);
    cmd.env("PALWORLD_LIFECYCLE_LOCK", lock);
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("\"ok\":true"));
}

#[test]
fn status_json_reports_success_for_running_container() {
    let name = unique_name("running-status");
    let lock = unique_lock();
    let container = TestContainer::create(name.clone());
    container.start();

    let mut cmd = Command::cargo_bin("palworld").expect("binary should build");
    cmd.args(["--output", "json", "status", "--name", &name]);
    cmd.env("PALWORLD_LIFECYCLE_LOCK", lock);
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("\"ok\":true"))
        .stdout(predicate::str::contains("\"running\":true"));
}

#[test]
fn logs_json_reports_content_for_existing_container() {
    let name = unique_name("logs-success");
    let lock = unique_lock();
    let container = TestContainer::create(name.clone());
    container.start();

    let mut cmd = Command::cargo_bin("palworld").expect("binary should build");
    cmd.args(["--output", "json", "logs", "--name", &name, "--tail", "20"]);
    cmd.env("PALWORLD_LIFECYCLE_LOCK", lock);
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("\"ok\":true"))
        .stdout(predicate::str::contains("ready"));
}

#[test]
fn container_start_json_succeeds_for_existing_stopped_container() {
    let name = unique_name("start-success");
    let lock = unique_lock();
    let _container = TestContainer::create(name.clone());

    let mut cmd = Command::cargo_bin("palworld").expect("binary should build");
    cmd.args(["--output", "json", "container-start", "--name", &name]);
    cmd.env("PALWORLD_LIFECYCLE_LOCK", lock);
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("\"ok\":true"));
}

#[test]
fn container_restart_json_succeeds_for_running_container() {
    let name = unique_name("restart-success");
    let lock = unique_lock();
    let container = TestContainer::create(name.clone());
    container.start();

    let mut cmd = Command::cargo_bin("palworld").expect("binary should build");
    cmd.args(["--output", "json", "container-restart", "--name", &name]);
    cmd.env("PALWORLD_LIFECYCLE_LOCK", lock);
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("\"ok\":true"));
}

#[test]
fn monitor_command_starts_without_webhook_and_can_be_terminated() {
    let bin = assert_cmd::cargo::cargo_bin("palworld");
    let mut child = StdCommand::new(bin)
        .args(["monitor", "--no-stream"])
        .spawn()
        .expect("monitor should start");
    thread::sleep(Duration::from_secs(1));
    child.kill().expect("monitor process should be killable");
    let _ = child.wait().expect("monitor process should exit");
}

#[test]
fn monitor_command_starts_with_discord_webhook_and_can_be_terminated() {
    let bin = assert_cmd::cargo::cargo_bin("palworld");
    let mut child = StdCommand::new(bin)
        .args(["monitor", "--no-stream"])
        .env(
            "WEBHOOK_URL",
            "https://discord.com/api/webhooks/1234567890/abcdefghijklmnopqrstuvwxyz",
        )
        .env("SERVER_NAME", "Coverage Test Server")
        .spawn()
        .expect("monitor should start");
    thread::sleep(Duration::from_secs(1));
    child.kill().expect("monitor process should be killable");
    let _ = child.wait().expect("monitor process should exit");
}
