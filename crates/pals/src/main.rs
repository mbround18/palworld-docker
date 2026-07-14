mod cli;
mod environment;
mod game_settings;
mod lifecycle;
mod runtime;
mod utils;
mod webhook;

use crate::cli::{emit_error, emit_success, ErrorCode, OutputMode};
use crate::environment::name;
use crate::lifecycle::guard::OperationGuard;
use crate::lifecycle::LifecycleManager;
use crate::runtime::docker::DockerCliRuntime;
use crate::runtime::{ContainerSpec, HealthStatus};
use clap::{Parser, Subcommand};
use gsm_cron::{begin_cron_loop, register_job};
use gsm_instance::{Instance, InstanceConfig};
use gsm_monitor::LogRules;
use gsm_notifications::notifications::{StandardServerEvents, send_notifications};
use gsm_shared::{fetch_var, is_env_var_truthy};
use serde_json::{json, Value};
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::{exit, Child, Command, Stdio};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;
use tracing::{debug, error, info, warn};

#[derive(Parser)]
#[command(name = "palworld", version = "1.0", about = "Manage Palworld Server")]
struct Cli {
    #[arg(long, value_enum, global = true, default_value_t = OutputMode::Human)]
    output: OutputMode,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        #[arg(long, default_value = "/home/steam/palworld")]
        path: PathBuf,
    },
    Start,
    Monitor {
        #[arg(long)]
        update_job: bool,
        #[arg(long, default_value_t = false)]
        no_stream: bool,
    },
    Stop,
    Restart,
    Update {
        #[arg(long)]
        check: bool,
    },
    Provision {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        image: Option<String>,
    },
    Status {
        #[arg(long)]
        name: Option<String>,
    },
    Logs {
        #[arg(long)]
        name: Option<String>,
        #[arg(long, default_value_t = 200)]
        tail: u32,
        #[arg(long)]
        follow: bool,
    },
    Remove {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        force: bool,
    },
    ContainerUpdate {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        image: Option<String>,
    },
    ContainerStart {
        #[arg(long)]
        name: Option<String>,
    },
    ContainerStop {
        #[arg(long)]
        name: Option<String>,
    },
    ContainerRestart {
        #[arg(long)]
        name: Option<String>,
    },
}

fn default_container_spec(name: Option<String>, image: Option<String>) -> ContainerSpec {
    let container_name = name
        .or_else(|| env::var("CONTAINER_NAME").ok())
        .unwrap_or_else(|| "palworld".to_owned());
    let container_image = image
        .or_else(|| env::var("PALWORLD_IMAGE").ok())
        .unwrap_or_else(|| "mbround18/palworld-docker:latest".to_owned());
    let data_path = env::var("PALWORLD_DATA_PATH").unwrap_or_else(|_| "/home/steam/palworld".to_owned());
    let mut env_vars = Vec::new();
    for key in [
        "PRESET",
        "SERVER_NAME",
        "SERVER_DESCRIPTION",
        "PUBLIC_IP",
        "PUBLIC_PORT",
        "PORT",
        "ADMIN_PASSWORD",
        "SERVER_PASSWORD",
        "REGION",
        "USE_AUTH",
        "WEBHOOK_URL",
        "MULTITHREADING",
        "AUTO_UPDATE",
        "AUTO_UPDATE_SCHEDULE",
    ] {
        if let Ok(value) = env::var(key) {
            env_vars.push(format!("{key}={value}"));
        }
    }

    ContainerSpec {
        name: container_name,
        image: container_image,
        ports: vec!["8211:8211/udp".to_owned(), "27015:27015/udp".to_owned()],
        volumes: vec![format!("{data_path}:/home/steam/palworld")],
        env: env_vars,
    }
}

fn monitor_log_paths(working_dir: &Path) -> (PathBuf, PathBuf) {
    (
        working_dir.join("logs/server.log"),
        working_dir.join("logs/server.err"),
    )
}

fn spawn_tail_stream(path: &Path) -> Result<Child, std::io::Error> {
    Command::new("tail")
        .arg("-n")
        .arg("+1")
        .arg("-F")
        .arg(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
}

fn lifecycle_lock_path() -> PathBuf {
    env::var("PALWORLD_LIFECYCLE_LOCK")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/palworld-lifecycle.lock"))
}

fn acquire_operation_guard(output: OutputMode) -> OperationGuard {
    let lock_path = lifecycle_lock_path();
    match OperationGuard::acquire(&lock_path) {
        Ok(guard) => guard,
        Err(error) => {
            emit_error(output, ErrorCode::Conflict, error.to_string());
            exit(3);
        }
    }
}

fn status_payload(container: &str, status: &crate::runtime::ContainerStatus) -> Value {
    json!({
        "container": container,
        "exists": status.exists,
        "running": status.running,
        "health": format!("{:?}", status.health).to_lowercase(),
    })
}

fn lifecycle_payload(container: &str, status: &crate::runtime::ContainerStatus) -> Value {
    json!({
        "container": container,
        "running": status.running,
        "health": format!("{:?}", status.health).to_lowercase(),
    })
}

fn handle_status_result(
    output_mode: OutputMode,
    container: &str,
    result: Result<crate::runtime::ContainerStatus, crate::runtime::RuntimeError>,
) -> i32 {
    match result {
        Ok(status) => {
            emit_success(
                output_mode,
                format!("Container '{}' status", container),
                Some(status_payload(container, &status)),
            );
            if status.exists && status.running && status.health == HealthStatus::Unhealthy {
                emit_error(
                    output_mode,
                    ErrorCode::Unhealthy,
                    format!("Container '{}' is unhealthy", container),
                );
                return 1;
            }
            if !status.exists {
                emit_error(
                    output_mode,
                    ErrorCode::NotFound,
                    format!("Container '{}' does not exist", container),
                );
                return 2;
            }
            0
        }
        Err(e) => {
            emit_error(
                output_mode,
                ErrorCode::RuntimeFailure,
                format!("Failed to inspect container '{}': {e}", container),
            );
            1
        }
    }
}

fn handle_mutation_result(
    output_mode: OutputMode,
    action: &str,
    container: &str,
    result: Result<crate::runtime::ContainerStatus, crate::runtime::RuntimeError>,
) -> i32 {
    match result {
        Ok(status) => {
            emit_success(
                output_mode,
                format!("Container '{}' {}", container, action),
                Some(lifecycle_payload(container, &status)),
            );
            0
        }
        Err(e) => {
            emit_error(
                output_mode,
                ErrorCode::RuntimeFailure,
                format!("Failed to {action} container '{}': {e}", container),
            );
            1
        }
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn default_container_spec_uses_defaults_when_env_is_missing() {
        let _lock = env_lock().lock().expect("env lock should work");
        for key in ["CONTAINER_NAME", "PALWORLD_IMAGE", "PALWORLD_DATA_PATH", "PRESET"] {
            unsafe { env::remove_var(key) };
        }

        let spec = default_container_spec(None, None);
        assert_eq!(spec.name, "palworld");
        assert_eq!(spec.image, "mbround18/palworld-docker:latest");
        assert_eq!(spec.volumes, vec!["/home/steam/palworld:/home/steam/palworld"]);
        assert!(spec.env.is_empty());
    }

    #[test]
    fn default_container_spec_uses_explicit_args() {
        let _lock = env_lock().lock().expect("env lock should work");
        let spec = default_container_spec(
            Some("custom-name".to_owned()),
            Some("example/palworld:test".to_owned()),
        );
        assert_eq!(spec.name, "custom-name");
        assert_eq!(spec.image, "example/palworld:test");
    }

    #[test]
    fn lifecycle_lock_path_uses_default_and_env_override() {
        let _lock = env_lock().lock().expect("env lock should work");
        unsafe { env::remove_var("PALWORLD_LIFECYCLE_LOCK") };
        assert_eq!(
            lifecycle_lock_path(),
            PathBuf::from("/tmp/palworld-lifecycle.lock")
        );

        unsafe { env::set_var("PALWORLD_LIFECYCLE_LOCK", "/tmp/custom.lock") };
        assert_eq!(lifecycle_lock_path(), PathBuf::from("/tmp/custom.lock"));
        unsafe { env::remove_var("PALWORLD_LIFECYCLE_LOCK") };
    }

    #[test]
    fn status_payload_includes_fields() {
        let status = crate::runtime::ContainerStatus {
            exists: true,
            running: true,
            health: HealthStatus::Healthy,
        };
        let payload = status_payload("palworld", &status);
        assert_eq!(payload["container"], "palworld");
        assert_eq!(payload["exists"], true);
        assert_eq!(payload["running"], true);
        assert_eq!(payload["health"], "healthy");
    }

    #[test]
    fn handle_status_result_reports_not_found() {
        let code = handle_status_result(
            OutputMode::Json,
            "palworld",
            Ok(crate::runtime::ContainerStatus::not_found()),
        );
        assert_eq!(code, 2);
    }

    #[test]
    fn handle_status_result_reports_unhealthy() {
        let code = handle_status_result(
            OutputMode::Json,
            "palworld",
            Ok(crate::runtime::ContainerStatus {
                exists: true,
                running: true,
                health: HealthStatus::Unhealthy,
            }),
        );
        assert_eq!(code, 1);
    }

    #[test]
    fn handle_status_result_reports_runtime_failure() {
        let code = handle_status_result(
            OutputMode::Json,
            "palworld",
            Err(crate::runtime::RuntimeError::Parse("boom".to_owned())),
        );
        assert_eq!(code, 1);
    }

    #[test]
    fn handle_mutation_result_success_is_zero() {
        let code = handle_mutation_result(
            OutputMode::Json,
            "updated",
            "palworld",
            Ok(crate::runtime::ContainerStatus {
                exists: true,
                running: false,
                health: HealthStatus::None,
            }),
        );
        assert_eq!(code, 0);
    }

    #[test]
    fn handle_mutation_result_failure_is_one() {
        let code = handle_mutation_result(
            OutputMode::Json,
            "updated",
            "palworld",
            Err(crate::runtime::RuntimeError::Parse("failed".to_owned())),
        );
        assert_eq!(code, 1);
    }

    #[test]
    fn monitor_log_paths_builds_expected_targets() {
        let (log_path, err_path) = monitor_log_paths(Path::new("/home/steam/palworld"));
        assert_eq!(
            log_path,
            PathBuf::from("/home/steam/palworld/logs/server.log")
        );
        assert_eq!(
            err_path,
            PathBuf::from("/home/steam/palworld/logs/server.err")
        );
    }
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    debug!("Tracing subscriber initialized.");

    let cli = Cli::parse();
    let output_mode = cli.output;
    let instance_config = InstanceConfig {
        app_id: 2_394_010, // Palworld Steam App ID
        name: name(),
        command: "/bin/bash".to_owned(),
        install_args: vec![],
        launch_args: {
            let mut args = vec!["./PalServer.sh".to_owned()];

            if let Ok(public_ip) = env::var("PUBLIC_IP") {
                args.push(format!("-publicip={public_ip}"));
            }

            if let Some(public_port) = env::var("PORT").ok().or_else(|| Some("8211".to_owned())) {
                args.push(format!("-port={public_port}"));
            }

            if let Some(public_port) = env::var("PUBLIC_PORT").ok().or_else(|| Some("8211".to_owned())) {
                args.push(format!("-publicport={public_port}"));
            }

            if is_env_var_truthy("PUBLIC_LOBBY") {
                args.push("-publiclobby".to_owned());
            }

            if is_env_var_truthy("MULTITHREADING") {
                args.push("-useperfthreads".to_owned());
                args.push("-NoAsyncLoadingThread".to_owned());
                args.push("-UseMultithreadForDS".to_owned());
            }

            args
        },
        force_windows: false,
        launch_mode: gsm_instance::config::LaunchMode::Native,
        working_dir: PathBuf::from("/home/steam/palworld"),
    };
    debug!("Instance configuration set: {:?}", instance_config);

    let instance = Arc::new(TokioMutex::new(Instance::new(instance_config)));
    debug!("Instance created and wrapped in Arc<Mutex<>>");

    match cli.command {
        Commands::Install { path } => {
            let _guard = acquire_operation_guard(output_mode);
            info!("Installing Palworld server to: {:?}", path);
            let inst = instance.lock().await;
            if let Err(e) = inst.install() {
                emit_error(output_mode, ErrorCode::RuntimeFailure, format!("Installation failed: {e}"));
                exit(1);
            } else {
                debug!("Installation successful.");
                let config_path = path.join("Pal/Saved/Config/LinuxServer/PalWorldSettings.ini");
                game_settings::load_or_create_config(&config_path);
                emit_success(output_mode, "Installation completed", None);
            }
        }
        Commands::Start => {
            let _guard = acquire_operation_guard(output_mode);
            info!("Starting server...");
            let inst = instance.lock().await;
            if let Err(e) = inst.start() {
                emit_error(output_mode, ErrorCode::RuntimeFailure, format!("Failed to start server: {e}"));
                exit(1);
            } else {
                emit_success(output_mode, "Server started", None);
            }
        }
        Commands::Monitor {
            update_job,
            no_stream,
        } => {
            let working_dir = {
                let inst = instance.lock().await;
                inst.config.working_dir.clone()
            };
            let (server_log_path, server_err_path) = monitor_log_paths(&working_dir);

            let mut _stream_children = Vec::new();
            if !no_stream {
                for path in [&server_log_path, &server_err_path] {
                    match spawn_tail_stream(path) {
                        Ok(child) => {
                            info!("Streaming log file to stdout: {}", path.display());
                            _stream_children.push(child);
                        }
                        Err(error) => {
                            warn!("Failed to stream log file {}: {}", path.display(), error);
                        }
                    }
                }
            }

            let rules = LogRules::default();

            if let Ok(webhook_url) = env::var("WEBHOOK_URL") {
                let server_name = env::var("SERVER_NAME").unwrap_or_else(|_| "Palworld Server".to_owned());
                let processor = webhook::ProcessorRegistry::create(webhook_url, server_name).map(Arc::new);
                let startup_state = Arc::new(StdMutex::new(webhook::StartupState::default()));
                let shutdown_in_progress = Arc::new(StdMutex::new(false));

                rules.add_rule(
                    |line| {
                        line == "Shutdown handler: initialize."
                            || line.contains("+++UE5+Release-")
                            || line.starts_with("Game version is ")
                            || line.contains("Running Palworld dedicated server on")
                    },
                    {
                        let startup_state = Arc::clone(&startup_state);
                        let shutdown_in_progress = Arc::clone(&shutdown_in_progress);
                        let processor = processor.clone();
                        move |line| {
                            let Ok(mut state) = startup_state.lock() else {
                                warn!("Failed to lock startup webhook state.");
                                return;
                            };
                            state.update_from_line(line);
                            let Some(event) = state.build_started_event(line) else {
                                return;
                            };

                            if let Some(processor) = &processor {
                                if let Err(error) = processor.send_server_event(&event) {
                                    warn!("Failed to send Discord startup notification: {error}");
                                }
                            } else if let Err(error) = send_notifications(StandardServerEvents::Started) {
                                warn!("Failed to send webhook startup notification: {error}");
                            }
                            if let Ok(mut is_shutting_down) = shutdown_in_progress.lock() {
                                *is_shutting_down = false;
                            } else {
                                warn!("Failed to lock shutdown webhook state.");
                            }
                            state.reset();
                        }
                    },
                    false,
                    None,
                );

                rules.add_rule(
                    |line| line.contains("Stopping Palworld server...") || line == "Server stopped",
                    {
                        let processor = processor.clone();
                        let shutdown_in_progress = Arc::clone(&shutdown_in_progress);
                        move |line| {
                            let Some(event) = webhook::parse_shutdown_event(line) else {
                                return;
                            };
                            let Ok(mut is_shutting_down) = shutdown_in_progress.lock() else {
                                warn!("Failed to lock shutdown webhook state.");
                                return;
                            };

                            match event.event_type {
                                webhook::ServerEventType::Stopping => {
                                    if *is_shutting_down {
                                        return;
                                    }
                                    *is_shutting_down = true;
                                }
                                webhook::ServerEventType::Stopped => {
                                    if !*is_shutting_down {
                                        return;
                                    }
                                    *is_shutting_down = false;
                                }
                                webhook::ServerEventType::Started => {}
                            }

                            if let Some(processor) = &processor {
                                if let Err(error) = processor.send_server_event(&event) {
                                    warn!("Failed to send Discord shutdown notification: {error}");
                                }
                            } else {
                                let standard_event = match event.event_type {
                                    webhook::ServerEventType::Stopping
                                    | webhook::ServerEventType::Stopped => StandardServerEvents::Stopped,
                                    webhook::ServerEventType::Started => StandardServerEvents::Started,
                                };
                                if let Err(error) = send_notifications(standard_event) {
                                    warn!("Failed to send webhook shutdown notification: {error}");
                                }
                            }
                        }
                    },
                    false,
                    None,
                );

                if let Some(processor) = processor {
                    rules.add_rule(
                        |line| line.contains("connected the server."),
                        {
                            let processor = Arc::clone(&processor);
                            move |line| {
                                if let Some(event) = webhook::parse_player_event(line)
                                    && let Err(error) = processor.send_player_event(&event)
                                {
                                    warn!("Failed to send Discord connected notification: {error}");
                                }
                            }
                        },
                        false,
                        None,
                    );
                    rules.add_rule(
                        |line| line.contains("joined the server."),
                        {
                            let processor = Arc::clone(&processor);
                            move |line| {
                                if let Some(event) = webhook::parse_player_event(line)
                                    && let Err(error) = processor.send_player_event(&event)
                                {
                                    warn!("Failed to send Discord joined notification: {error}");
                                }
                            }
                        },
                        false,
                        None,
                    );
                    rules.add_rule(
                        |line| line.contains("left the server."),
                        {
                            let processor = Arc::clone(&processor);
                            move |line| {
                                if let Some(event) = webhook::parse_player_event(line)
                                    && let Err(error) = processor.send_player_event(&event)
                                {
                                    warn!("Failed to send Discord left notification: {error}");
                                }
                            }
                        },
                        false,
                        None,
                    );
                } else {
                    warn!("WEBHOOK_URL is set but no supported processor matched it.");
                }
            }

            gsm_monitor::start_instance_log_monitor(&working_dir, rules);

            if update_job || is_env_var_truthy("AUTO_UPDATE") {
                let update_schedule = fetch_var("AUTO_UPDATE_SCHEDULE", "0 3 * * *");
                let instance_clone = Arc::clone(&instance);
                register_job("auto-update", &update_schedule, move || {
                    let instance_clone_inner = Arc::clone(&instance_clone);
                    tokio::spawn(async move {
                        let inst = instance_clone_inner.lock().await;
                        if inst.update_available() {
                            warn!("Update available! Stopping server...");
                            if let Err(e) = inst.stop() {
                                error!("Failed to stop server: {}", e);
                                return;
                            }
                            info!("Updating server...");
                            if let Err(e) = inst.update() {
                                error!("Update failed: {}", e);
                                return;
                            }
                            info!("Restarting server...");
                            if let Err(e) = inst.start() {
                                error!("Failed to start server: {}", e);
                            }
                        }
                    });
                });
            }

            debug!("Entering cron loop (monitoring logs and scheduled tasks)...");
            begin_cron_loop().await;
        }
        Commands::Stop => {
            let _guard = acquire_operation_guard(output_mode);
            warn!("Stopping Palworld server...");
            let inst = instance.lock().await;
            if let Err(e) = inst.stop() {
                emit_error(output_mode, ErrorCode::RuntimeFailure, format!("Failed to stop: {e}"));
                exit(1);
            } else {
                if env::var("WEBHOOK_URL").is_ok()
                    && let Err(e) = send_notifications(StandardServerEvents::Stopped)
                {
                    warn!("Failed to send webhook notification: {e}");
                }
                debug!("Server stopped successfully.");
                emit_success(output_mode, "Server stopped", None);
            }
        }
        Commands::Restart => {
            let _guard = acquire_operation_guard(output_mode);
            warn!("Restarting Palworld server...");
            let inst = instance.lock().await;
            if let Err(e) = inst.restart() {
                emit_error(output_mode, ErrorCode::RuntimeFailure, format!("Failed to restart server: {e}"));
                exit(1);
            } else {
                emit_success(output_mode, "Server restarted", None);
            }
        }
        Commands::Update { check } => {
            let _guard = acquire_operation_guard(output_mode);
            let inst = instance.lock().await;
            if check {
                if inst.update_available() {
                    emit_error(output_mode, ErrorCode::RuntimeFailure, "Update available");
                    exit(1);
                } else {
                    emit_success(output_mode, "Server is up to date", None);
                    exit(0);
                }
            } else if inst.update_available() {
                warn!("Update available! Updating...");
                if let Err(e) = inst.update() {
                    emit_error(output_mode, ErrorCode::RuntimeFailure, format!("Update failed: {e}"));
                    exit(1);
                } else {
                    emit_success(output_mode, "Server updated", None);
                }
            } else {
                emit_success(output_mode, "No update available", None);
            }
        }
        Commands::Provision { name, image } => {
            let _guard = acquire_operation_guard(output_mode);
            let lifecycle = LifecycleManager::new(DockerCliRuntime);
            let spec = default_container_spec(name, image);
            let exit_code = handle_mutation_result(
                output_mode,
                "provisioned",
                &spec.name,
                lifecycle.provision(&spec),
            );
            if exit_code != 0 {
                exit(exit_code);
            }
        }
        Commands::Status { name } => {
            let lifecycle = LifecycleManager::new(DockerCliRuntime);
            let spec = default_container_spec(name, None);
            let exit_code = handle_status_result(output_mode, &spec.name, lifecycle.status(&spec.name));
            if exit_code != 0 {
                exit(exit_code);
            }
        }
        Commands::Logs { name, tail, follow } => {
            let lifecycle = LifecycleManager::new(DockerCliRuntime);
            let spec = default_container_spec(name, None);
            match lifecycle.logs(&spec.name, follow, tail) {
                Ok(logs) => {
                    if output_mode == OutputMode::Json {
                        emit_success(
                            output_mode,
                            format!("Container '{}' logs", spec.name),
                            Some(serde_json::json!({ "container": spec.name, "logs": logs })),
                        );
                    } else {
                        println!("{logs}");
                    }
                }
                Err(e) => {
                    emit_error(
                        output_mode,
                        ErrorCode::RuntimeFailure,
                        format!("Failed to fetch logs from '{}': {e}", spec.name),
                    );
                    exit(1);
                }
            }
        }
        Commands::Remove { name, force } => {
            let _guard = acquire_operation_guard(output_mode);
            let lifecycle = LifecycleManager::new(DockerCliRuntime);
            let spec = default_container_spec(name, None);
            if let Err(e) = lifecycle.remove(&spec.name, force) {
                emit_error(
                    output_mode,
                    ErrorCode::RuntimeFailure,
                    format!("Failed to remove container '{}': {e}", spec.name),
                );
                exit(1);
            }
            emit_success(
                output_mode,
                format!("Container '{}' removed", spec.name),
                Some(serde_json::json!({ "container": spec.name, "force": force })),
            );
        }
        Commands::ContainerUpdate { name, image } => {
            let _guard = acquire_operation_guard(output_mode);
            let lifecycle = LifecycleManager::new(DockerCliRuntime);
            let spec = default_container_spec(name, image);
            let exit_code = handle_mutation_result(
                output_mode,
                "updated",
                &spec.name,
                lifecycle.update(&spec),
            );
            if exit_code != 0 {
                exit(exit_code);
            }
        }
        Commands::ContainerStart { name } => {
            let _guard = acquire_operation_guard(output_mode);
            let lifecycle = LifecycleManager::new(DockerCliRuntime);
            let spec = default_container_spec(name, None);
            let exit_code = handle_mutation_result(
                output_mode,
                "started",
                &spec.name,
                lifecycle.start(&spec.name),
            );
            if exit_code != 0 {
                exit(exit_code);
            }
        }
        Commands::ContainerStop { name } => {
            let _guard = acquire_operation_guard(output_mode);
            let lifecycle = LifecycleManager::new(DockerCliRuntime);
            let spec = default_container_spec(name, None);
            let exit_code = handle_mutation_result(
                output_mode,
                "stopped",
                &spec.name,
                lifecycle.stop(&spec.name),
            );
            if exit_code != 0 {
                exit(exit_code);
            }
        }
        Commands::ContainerRestart { name } => {
            let _guard = acquire_operation_guard(output_mode);
            let lifecycle = LifecycleManager::new(DockerCliRuntime);
            let spec = default_container_spec(name, None);
            let exit_code = handle_mutation_result(
                output_mode,
                "restarted",
                &spec.name,
                lifecycle.restart(&spec.name),
            );
            if exit_code != 0 {
                exit(exit_code);
            }
        }
    }
}
