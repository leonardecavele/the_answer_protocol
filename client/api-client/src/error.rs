use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct CommandError {
    pub code: Option<i32>,
    pub message: String,
}
impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(c) = self.code {
            write!(f, "{} (code: {})", self.message, c)
        } else {
            write!(f, "{}", self.message)
        }
    }
}
impl std::error::Error for CommandError {}

impl CommandError {
    pub fn with_message(&mut self, message: Option<String>) {
        if let Some(m) = message {
            self.message = m;
        }
    }

    pub fn default_message_from_code(code_opt: Option<i32>) -> String {
        let code = code_opt.unwrap_or(-1);

        match code {
            400 => String::from("bad request"),
            401 => String::from("unauthorized"),
            403 => String::from("forbidden"),
            404 => String::from("not found"),
            405 => String::from("method not allowed"),
            406 => String::from("not acceptable"),
            408 => String::from("request timeout"),
            409 => String::from("conflict"),
            410 => String::from("gone"),
            422 => String::from("unprocessable entity"),
            429 => String::from("too many requests"),
            500 => String::from("internal server error"),
            501 => String::from("not implemented"),
            502 => String::from("bad gateway"),
            503 => String::from("service unavailable"),
            504 => String::from("gateway timeout"),
            900 => String::from("server unavailable"),
            _ => "unknown server error".to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("codec error: {0}")]
    Codec(#[from] tokio_util::codec::LinesCodecError),

    #[error("could not connect to {addr}: {source}")]
    ConnectionFailed {
        addr: String,
        source: std::io::Error,
    },

    #[error("could not connect to {addr} within {}s", timeout.as_secs())]
    ConnectionTimeout { addr: String, timeout: Duration },

    #[error("the server did not answer within {}s (command: {command})", timeout.as_secs())]
    RequestTimeout { command: String, timeout: Duration },

    #[error("the server did not handshake within {}s", timeout.as_secs())]
    HandshakeTimeout { timeout: Duration },

    #[error("connection disconnected unexpectedly")]
    Disconnected,
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("invalid opcode: expected one of {expected}, got '{received}'")]
    InvalidOpcode { expected: String, received: String },

    #[error("invalid arguments: expected {expected}, got '{received}'")]
    InvalidArguments { expected: String, received: String },

    #[error("unsupported protocol version {server}, require {supported}")]
    UnsupportedVersion { server: u32, supported: u32 },

    #[error("could not parse the server response: {0}")]
    Parse(String),
}

#[derive(Error, Debug)]
pub enum InternalError {
    #[error("{0}")]
    BridgeUnavailable(String),
}

#[derive(Error, Debug)]
pub enum TapError {
    #[error(transparent)]
    Network(#[from] NetworkError),

    #[error(transparent)]
    Protocol(#[from] ProtocolError),

    #[error(transparent)]
    Internal(#[from] InternalError),
}
