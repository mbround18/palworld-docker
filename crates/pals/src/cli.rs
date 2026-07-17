use clap::ValueEnum;
use serde::Serialize;
use serde_json::Value;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputMode {
    Human,
    Json,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    RuntimeFailure,
    Conflict,
    NotFound,
    Unhealthy,
}

#[derive(Debug, Serialize)]
pub struct CommandEnvelope {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<ErrorCode>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

pub fn format_success(mode: OutputMode, message: impl Into<String>, data: Option<Value>) -> String {
    let message = message.into();
    match mode {
        OutputMode::Human => message,
        OutputMode::Json => serde_json::to_string(&CommandEnvelope {
            ok: true,
            code: None,
            message,
            data,
        })
        .unwrap_or_else(|_| "{\"ok\":true,\"message\":\"serialization failed\"}".to_owned()),
    }
}

pub fn format_error(mode: OutputMode, code: ErrorCode, message: impl Into<String>) -> String {
    let message = message.into();
    match mode {
        OutputMode::Human => format!("[{:?}] {message}", code),
        OutputMode::Json => serde_json::to_string(&CommandEnvelope {
            ok: false,
            code: Some(code),
            message,
            data: None,
        })
        .unwrap_or_else(|_| {
            "{\"ok\":false,\"code\":\"runtime_failure\",\"message\":\"serialization failed\"}"
                .to_owned()
        }),
    }
}

pub fn emit_success(mode: OutputMode, message: impl Into<String>, data: Option<Value>) {
    println!("{}", format_success(mode, message, data));
}

pub fn emit_error(mode: OutputMode, code: ErrorCode, message: impl Into<String>) {
    eprintln!("{}", format_error(mode, code, message));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_json_success_envelope_with_data() {
        let output = format_success(
            OutputMode::Json,
            "container started",
            Some(serde_json::json!({"running": true})),
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("success json should parse");
        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["message"], "container started");
        assert_eq!(parsed["data"]["running"], true);
    }

    #[test]
    fn formats_json_error_envelope_with_code() {
        let output = format_error(OutputMode::Json, ErrorCode::Conflict, "operation already active");
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("error json should parse");
        assert_eq!(parsed["ok"], false);
        assert_eq!(parsed["code"], "conflict");
        assert_eq!(parsed["message"], "operation already active");
    }
}

