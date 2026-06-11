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

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("codec error: {0}")]
    Codec(#[from] tokio_util::codec::LinesCodecError),

    #[error("connection to server failed")]
    ConnectionMaxRetry,

    #[error("connection to server timed out")]
    ConnectionTimeout,

    #[error("connection disconnected unexpectedly")]
    Disconnected,
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("invalid opcode. expected ({expected}), received '{received}'")]
    InvalidOpcode { expected: String, received: String },

    #[error("invalid arguments. expected ({expected}), received '{received}'")]
    InvalidArguments { expected: String, received: String },

    #[error("response parse error: {0}")]
    Parse(String),
}

#[derive(Error, Debug)]
pub enum InternalError {
    #[error("channel communication error: {0}")]
    ChannelPanic(String),

    #[error("thread panic: {0}")]
    ThreadPanic(String),
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
