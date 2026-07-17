use regex::Regex;
use reqwest::blocking::{Client, Response};
use reqwest::header::RETRY_AFTER;
use serde_json::json;
use std::sync::LazyLock;
use std::thread::sleep;
use std::time::Duration;

#[allow(clippy::expect_used)]
static CONNECTED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[(?P<ts>[^\]]+)\]\s+\[LOG\]\s+(?P<name>\S+)\s+\S+\s+connected the server\.")
        .expect("connected regex should compile")
});

#[allow(clippy::expect_used)]
static JOINED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[(?P<ts>[^\]]+)\]\s+\[LOG\]\s+(?P<name>\S+)\s+joined the server\.")
        .expect("joined regex should compile")
});

#[allow(clippy::expect_used)]
static LEFT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[(?P<ts>[^\]]+)\]\s+\[LOG\]\s+(?P<name>\S+)\s+left the server\.")
        .expect("left regex should compile")
});

#[allow(clippy::expect_used)]
static GAME_VERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^Game version is (?P<version>v[^\s]+)$").expect("game version regex should compile")
});

#[allow(clippy::expect_used)]
static STARTUP_ENDPOINT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^Running Palworld dedicated server on (?P<endpoint>.+)$")
        .expect("startup endpoint regex should compile")
});

#[allow(clippy::expect_used)]
static STOPPING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:(?P<ts>\S+)\s+)?WARN palworld: Stopping Palworld server\.\.\.$")
        .expect("stopping regex should compile")
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerEventType {
    Connected,
    Joined,
    Left,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerEvent {
    pub event_type: PlayerEventType,
    pub timestamp: String,
    pub player_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerEventType {
    Started,
    Stopping,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerEvent {
    pub event_type: ServerEventType,
    pub timestamp: Option<String>,
    pub engine_version: Option<String>,
    pub game_version: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StartupState {
    saw_shutdown_handler_init: bool,
    engine_version: Option<String>,
    game_version: Option<String>,
}

impl StartupState {
    pub fn update_from_line(&mut self, line: &str) {
        if line == "Shutdown handler: initialize." {
            self.saw_shutdown_handler_init = true;
            return;
        }

        if self.saw_shutdown_handler_init
            && self.engine_version.is_none()
            && line.contains("+++UE5+Release-")
        {
            self.engine_version = Some(line.to_owned());
            return;
        }

        if let Some(caps) = GAME_VERSION_RE.captures(line) {
            self.game_version = caps.name("version").map(|v| v.as_str().to_owned());
        }
    }

    pub fn build_started_event(&self, line: &str) -> Option<ServerEvent> {
        let caps = STARTUP_ENDPOINT_RE.captures(line)?;
        if !self.saw_shutdown_handler_init {
            return None;
        }

        Some(ServerEvent {
            event_type: ServerEventType::Started,
            timestamp: None,
            engine_version: self.engine_version.clone(),
            game_version: self.game_version.clone(),
            endpoint: caps.name("endpoint").map(|v| v.as_str().to_owned()),
        })
    }

    pub fn reset(&mut self) {
        self.saw_shutdown_handler_init = false;
        self.engine_version = None;
        self.game_version = None;
    }
}

#[derive(Debug)]
pub struct ProcessorError(pub String);

impl std::fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ProcessorError {}

pub trait NotificationProcessor: Send + Sync {
    fn send_player_event(&self, event: &PlayerEvent) -> Result<(), ProcessorError>;
    fn send_server_event(&self, event: &ServerEvent) -> Result<(), ProcessorError>;
}

pub struct ProcessorRegistry;

impl ProcessorRegistry {
    pub fn create(
        webhook_url: String,
        server_name: String,
    ) -> Option<Box<dyn NotificationProcessor>> {
        if webhook_url.contains("discord.com/api/webhooks")
            || webhook_url.contains("discordapp.com/api/webhooks")
        {
            return Some(Box::new(DiscordWebhookProcessor::new(
                webhook_url,
                server_name,
            )));
        }
        None
    }
}

pub struct DiscordWebhookProcessor {
    webhook_url: String,
    server_name: String,
    client: Client,
}

impl DiscordWebhookProcessor {
    pub fn new(webhook_url: String, server_name: String) -> Self {
        Self {
            webhook_url,
            server_name,
            client: Client::new(),
        }
    }

    fn event_title(event_type: PlayerEventType) -> &'static str {
        match event_type {
            PlayerEventType::Connected => "Player Connected",
            PlayerEventType::Joined => "Player Joined",
            PlayerEventType::Left => "Player Left",
        }
    }

    fn event_color(event_type: PlayerEventType) -> u32 {
        match event_type {
            PlayerEventType::Connected => 0x3498DB,
            PlayerEventType::Joined => 0x2ECC71,
            PlayerEventType::Left => 0xE74C3C,
        }
    }

    fn server_event_title(event_type: ServerEventType) -> &'static str {
        match event_type {
            ServerEventType::Started => "Server Started",
            ServerEventType::Stopping => "Server Stopping",
            ServerEventType::Stopped => "Server Stopped",
        }
    }

    fn server_event_color(event_type: ServerEventType) -> u32 {
        match event_type {
            ServerEventType::Started => 0x2ECC71,
            ServerEventType::Stopping => 0xF39C12,
            ServerEventType::Stopped => 0xE74C3C,
        }
    }

    fn retry_after_seconds(response: &Response) -> Option<u64> {
        response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
    }
}

impl NotificationProcessor for DiscordWebhookProcessor {
    fn send_player_event(&self, event: &PlayerEvent) -> Result<(), ProcessorError> {
        let payload = json!({
            "username": "Palworld Monitor",
            "embeds": [{
                "title": Self::event_title(event.event_type),
                "color": Self::event_color(event.event_type),
                "fields": [
                    {"name": "Player", "value": event.player_name, "inline": true},
                    {"name": "Time", "value": event.timestamp, "inline": true},
                    {"name": "Server", "value": self.server_name, "inline": true}
                ]
            }]
        });

        let mut attempts = 0_u8;
        loop {
            attempts = attempts.saturating_add(1);
            let response = self
                .client
                .post(&self.webhook_url)
                .json(&payload)
                .send()
                .map_err(|e| ProcessorError(format!("failed to send webhook request: {e}")))?;

            if response.status().is_success() || response.status().as_u16() == 204 {
                return Ok(());
            }

            if response.status().as_u16() == 429 && attempts < 3 {
                let retry_for = Self::retry_after_seconds(&response).unwrap_or(1);
                sleep(Duration::from_secs(retry_for));
                continue;
            }

            if response.status().is_server_error() && attempts < 3 {
                sleep(Duration::from_secs(1));
                continue;
            }

            return Err(ProcessorError(format!(
                "webhook returned status {}",
                response.status()
            )));
        }
    }

    fn send_server_event(&self, event: &ServerEvent) -> Result<(), ProcessorError> {
        let mut fields = vec![json!({"name": "Server", "value": self.server_name, "inline": true})];
        if let Some(timestamp) = &event.timestamp {
            fields.push(json!({"name": "Time", "value": timestamp, "inline": true}));
        }
        if let Some(game_version) = &event.game_version {
            fields.push(json!({"name": "Game Version", "value": game_version, "inline": true}));
        }
        if let Some(engine_version) = &event.engine_version {
            fields.push(json!({"name": "Engine Build", "value": engine_version, "inline": false}));
        }
        if let Some(endpoint) = &event.endpoint {
            fields.push(json!({"name": "Endpoint", "value": endpoint, "inline": true}));
        }

        let payload = json!({
            "username": "Palworld Monitor",
            "embeds": [{
                "title": Self::server_event_title(event.event_type),
                "color": Self::server_event_color(event.event_type),
                "fields": fields
            }]
        });

        let mut attempts = 0_u8;
        loop {
            attempts = attempts.saturating_add(1);
            let response = self
                .client
                .post(&self.webhook_url)
                .json(&payload)
                .send()
                .map_err(|e| ProcessorError(format!("failed to send webhook request: {e}")))?;

            if response.status().is_success() || response.status().as_u16() == 204 {
                return Ok(());
            }

            if response.status().as_u16() == 429 && attempts < 3 {
                let retry_for = Self::retry_after_seconds(&response).unwrap_or(1);
                sleep(Duration::from_secs(retry_for));
                continue;
            }

            if response.status().is_server_error() && attempts < 3 {
                sleep(Duration::from_secs(1));
                continue;
            }

            return Err(ProcessorError(format!(
                "webhook returned status {}",
                response.status()
            )));
        }
    }
}

pub fn parse_player_event(line: &str) -> Option<PlayerEvent> {
    if let Some(caps) = CONNECTED_RE.captures(line) {
        return Some(PlayerEvent {
            event_type: PlayerEventType::Connected,
            timestamp: caps.name("ts")?.as_str().to_owned(),
            player_name: caps.name("name")?.as_str().to_owned(),
        });
    }
    if let Some(caps) = JOINED_RE.captures(line) {
        return Some(PlayerEvent {
            event_type: PlayerEventType::Joined,
            timestamp: caps.name("ts")?.as_str().to_owned(),
            player_name: caps.name("name")?.as_str().to_owned(),
        });
    }
    if let Some(caps) = LEFT_RE.captures(line) {
        return Some(PlayerEvent {
            event_type: PlayerEventType::Left,
            timestamp: caps.name("ts")?.as_str().to_owned(),
            player_name: caps.name("name")?.as_str().to_owned(),
        });
    }
    None
}

pub fn parse_shutdown_event(line: &str) -> Option<ServerEvent> {
    if let Some(caps) = STOPPING_RE.captures(line) {
        return Some(ServerEvent {
            event_type: ServerEventType::Stopping,
            timestamp: caps.name("ts").map(|v| v.as_str().to_owned()),
            engine_version: None,
            game_version: None,
            endpoint: None,
        });
    }
    if line == "Server stopped" {
        return Some(ServerEvent {
            event_type: ServerEventType::Stopped,
            timestamp: None,
            engine_version: None,
            game_version: None,
            endpoint: None,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    fn spawn_webhook_server(
        responses: Vec<&'static str>,
    ) -> (String, std::sync::mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener.local_addr().expect("local addr should resolve");
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("accept should work");
                let mut buf = [0_u8; 8192];
                let read_bytes = stream.read(&mut buf).expect("read should work");
                tx.send(String::from_utf8_lossy(&buf[..read_bytes]).to_string())
                    .expect("send should work");
                stream
                    .write_all(response.as_bytes())
                    .expect("write should work");
            }
        });
        (format!("http://{addr}/discord"), rx)
    }

    #[test]
    fn parser_extracts_connected_without_ids_or_ip() {
        let line = "[2026-07-14 12:56:28] [LOG] mbround18 172.20.0.1 connected the server. (User id: steam_76561198028400651)";
        let event = parse_player_event(line).expect("line should parse");
        assert_eq!(event.event_type, PlayerEventType::Connected);
        assert_eq!(event.timestamp, "2026-07-14 12:56:28");
        assert_eq!(event.player_name, "mbround18");
    }

    #[test]
    fn parser_extracts_joined_without_user_or_player_id() {
        let line = "[2026-07-14 12:56:42] [LOG] mbround18 joined the server. (User id: steam_76561198028400651, Player id: 7828553D000000000000000000000000)";
        let event = parse_player_event(line).expect("line should parse");
        assert_eq!(event.event_type, PlayerEventType::Joined);
        assert_eq!(event.timestamp, "2026-07-14 12:56:42");
        assert_eq!(event.player_name, "mbround18");
    }

    #[test]
    fn parser_extracts_left_without_user_id() {
        let line = "[2026-07-14 12:59:15] [LOG] mbround18 left the server. (User id: steam_76561198028400651)";
        let event = parse_player_event(line).expect("line should parse");
        assert_eq!(event.event_type, PlayerEventType::Left);
        assert_eq!(event.timestamp, "2026-07-14 12:59:15");
        assert_eq!(event.player_name, "mbround18");
    }

    #[test]
    fn parser_ignores_unsupported_lines() {
        assert!(parse_player_event("random line").is_none());
    }

    #[test]
    fn registry_creates_discord_processor_for_discord_webhook_url() {
        let processor = ProcessorRegistry::create(
            "https://discord.com/api/webhooks/123/abc".to_owned(),
            "My Server".to_owned(),
        );
        assert!(processor.is_some());
    }

    #[test]
    fn registry_returns_none_for_unsupported_webhook_url() {
        let processor =
            ProcessorRegistry::create("https://hooks.slack.com/services/x/y/z".to_owned(), "S".to_owned());
        assert!(processor.is_none());
    }

    #[test]
    fn discord_processor_sends_privacy_safe_payload() {
        let (url, rx) = spawn_webhook_server(vec![
            "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n",
        ]);
        let processor = DiscordWebhookProcessor::new(url, "My Server".to_owned());
        let event = PlayerEvent {
            event_type: PlayerEventType::Joined,
            timestamp: "2026-07-14 12:56:42".to_owned(),
            player_name: "mbround18".to_owned(),
        };
        processor
            .send_player_event(&event)
            .expect("send should succeed");

        let request = rx.recv().expect("request should be captured");
        assert!(request.contains("mbround18"));
        assert!(request.contains("2026-07-14 12:56:42"));
        assert!(!request.contains("steam_"));
        assert!(!request.contains("Player id"));
    }

    #[test]
    fn discord_processor_retries_on_429_then_succeeds() {
        let (url, rx) = spawn_webhook_server(vec![
            "HTTP/1.1 429 Too Many Requests\r\nRetry-After: 0\r\nContent-Length: 0\r\n\r\n",
            "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n",
        ]);
        let processor = DiscordWebhookProcessor::new(url, "My Server".to_owned());
        let event = PlayerEvent {
            event_type: PlayerEventType::Connected,
            timestamp: "2026-07-14 12:56:28".to_owned(),
            player_name: "mbround18".to_owned(),
        };
        processor
            .send_player_event(&event)
            .expect("retry path should eventually succeed");

        let _first = rx.recv().expect("first request should be captured");
        let _second = rx.recv().expect("second request should be captured");
    }

    #[test]
    fn discord_processor_fails_on_terminal_400() {
        let (url, _rx) = spawn_webhook_server(vec![
            "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n",
        ]);
        let processor = DiscordWebhookProcessor::new(url, "My Server".to_owned());
        let event = PlayerEvent {
            event_type: PlayerEventType::Left,
            timestamp: "2026-07-14 12:59:15".to_owned(),
            player_name: "mbround18".to_owned(),
        };
        let error = processor.send_player_event(&event).expect_err("send should fail");
        assert!(error.to_string().contains("status"));
    }

    #[test]
    fn startup_state_emits_started_with_versions_after_init_sequence() {
        let mut state = StartupState::default();
        state.update_from_line("Shutdown handler: initialize.");
        state.update_from_line("5.1.1-0+++UE5+Release-5.1 1008 0");
        state.update_from_line("Game version is v1.0.0.100427");

        let event = state
            .build_started_event("Running Palworld dedicated server on :8211")
            .expect("started event should be emitted");

        assert_eq!(event.event_type, ServerEventType::Started);
        assert_eq!(event.game_version.as_deref(), Some("v1.0.0.100427"));
        assert_eq!(
            event.engine_version.as_deref(),
            Some("5.1.1-0+++UE5+Release-5.1 1008 0")
        );
        assert_eq!(event.endpoint.as_deref(), Some(":8211"));
    }

    #[test]
    fn startup_state_does_not_emit_started_without_shutdown_init() {
        let mut state = StartupState::default();
        state.update_from_line("Game version is v1.0.0.100427");
        assert!(
            state
                .build_started_event("Running Palworld dedicated server on :8211")
                .is_none()
        );
    }

    #[test]
    fn parse_shutdown_event_handles_stopping_and_stopped() {
        let stopping = parse_shutdown_event(
            "2026-07-14T20:31:58.946490Z  WARN palworld: Stopping Palworld server...",
        )
        .expect("stopping should parse");
        assert_eq!(stopping.event_type, ServerEventType::Stopping);
        assert_eq!(
            stopping.timestamp.as_deref(),
            Some("2026-07-14T20:31:58.946490Z")
        );

        let stopped = parse_shutdown_event("Server stopped").expect("stopped should parse");
        assert_eq!(stopped.event_type, ServerEventType::Stopped);
    }

    #[test]
    fn discord_processor_sends_started_payload_with_versions() {
        let (url, rx) = spawn_webhook_server(vec![
            "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n",
        ]);
        let processor = DiscordWebhookProcessor::new(url, "My Server".to_owned());
        let event = ServerEvent {
            event_type: ServerEventType::Started,
            timestamp: None,
            engine_version: Some("5.1.1-0+++UE5+Release-5.1 1008 0".to_owned()),
            game_version: Some("v1.0.0.100427".to_owned()),
            endpoint: Some(":8211".to_owned()),
        };
        processor
            .send_server_event(&event)
            .expect("send should succeed");

        let request = rx.recv().expect("request should be captured");
        assert!(request.contains("Server Started"));
        assert!(request.contains("v1.0.0.100427"));
        assert!(request.contains("5.1.1-0+++UE5+Release-5.1 1008 0"));
        assert!(request.contains(":8211"));
    }
}
