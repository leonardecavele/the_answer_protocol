use thiserror::Error;

#[derive(Error, Debug)]
pub enum TapError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("network codec error: {0}")]
    Codec(#[from] tokio_util::codec::LinesCodecError),

    #[error("server response parsing error: {0}")]
    ServerResponseParse(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("protocol error: invalid opcode. expected ({0}), received '{1}'")]
    ProtocolInvalidOpcode(String, String),

    #[error("protocol error: Invalid arguments. expected ({0}), received '{1}'")]
    ProtocolInvalidArguments(String, String),

    #[error("internal channel communication error: {0}")]
    Channel(String),

    #[error("internal thread error: {0}")]
    ThreadPanic(String),

    #[error("unexpected disconnection or connection reset")]
    Disconnected,
}

pub type TapResult<T> = Result<T, TapError>;
