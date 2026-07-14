use regex::Regex;
use std::sync::LazyLock;

#[allow(clippy::expect_used)]
static JOINED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[LOG\]\s+(\w+)\s+joined the server")
        .expect("joined-player regex should compile")
});

#[allow(clippy::expect_used)]
static LEFT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[LOG\]\s+(\w+)\s+left the server").expect("left-player regex should compile")
});

/// Extracts the player name from a log line.
///
/// The log line is expected to contain a timestamp, "[LOG]", then the player name
/// before "joined the server".
pub fn extract_player_joined_name(log: &str) -> Option<String> {
    JOINED_RE
        .captures(log)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()))
}

/// Extracts the player name from a log line when a player leaves.
///
/// The log line is expected to contain a timestamp, "[LOG]", then the player name
/// before "left the server".
pub fn extract_player_left_name(log: &str) -> Option<String> {
    LEFT_RE
        .captures(log)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joined_extracts_name_from_log_line() {
        let log = "[2024.01.01-00.00.00:000][  0]LogNet: [LOG] mbround18 joined the server";
        assert_eq!(
            extract_player_joined_name(log),
            Some("mbround18".to_owned())
        );
    }

    #[test]
    fn joined_returns_none_when_pattern_absent() {
        assert_eq!(extract_player_joined_name("[server] Some other log line"), None);
        assert_eq!(extract_player_joined_name(""), None);
    }

    #[test]
    fn left_extracts_name_from_log_line() {
        let log = "[2024.01.01-00.00.00:000][  0]LogNet: [LOG] mbround18 left the server";
        assert_eq!(
            extract_player_left_name(log),
            Some("mbround18".to_owned())
        );
    }

    #[test]
    fn left_returns_none_when_pattern_absent() {
        assert_eq!(extract_player_left_name("[server] Server started."), None);
        assert_eq!(extract_player_left_name(""), None);
    }
}
