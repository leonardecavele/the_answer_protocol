use std::time::Duration;
use thiserror::Error;

/// The error codes the gateway and the game server can answer with, as listed in the root
/// [TAP protocol reference](../../PROTOCOL.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    NameInUse = 201,
    NoContent = 204,
    NoExit = 301,
    BadRequest = 400,
    NotInGroup = 401,
    AlreadyInGroup = 402,
    Forbidden = 403,
    NotFound = 404,
    ActionNotAllowed = 405,
    NoQuestAvailable = 406,
    NotInSameRoom = 407,
    NpcInCombat = 408,
    ActionAlreadyTaken = 409,
    PlayerAlreadyInCombat = 410,
    PlayerNotInCombat = 411,
    FileNotFound = 412,
    RoomNotFound = 413,
    MissingItem = 414,
    NotUsable = 415,
    DataTooBig = 416,
    TooManyRequests = 429,
    ConnectionFailed = 900,
    SendFailed = 901,
    GameServerTimeout = 902,
    InvalidGroupCommand = 997,
    InvalidQuestion = 998,
    InvalidCommand = 999,
}

impl ErrorCode {
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            201 => Some(Self::NameInUse),
            204 => Some(Self::NoContent),
            301 => Some(Self::NoExit),
            400 => Some(Self::BadRequest),
            401 => Some(Self::NotInGroup),
            402 => Some(Self::AlreadyInGroup),
            403 => Some(Self::Forbidden),
            404 => Some(Self::NotFound),
            405 => Some(Self::ActionNotAllowed),
            406 => Some(Self::NoQuestAvailable),
            407 => Some(Self::NotInSameRoom),
            408 => Some(Self::NpcInCombat),
            409 => Some(Self::ActionAlreadyTaken),
            410 => Some(Self::PlayerAlreadyInCombat),
            411 => Some(Self::PlayerNotInCombat),
            412 => Some(Self::FileNotFound),
            413 => Some(Self::RoomNotFound),
            414 => Some(Self::MissingItem),
            415 => Some(Self::NotUsable),
            416 => Some(Self::DataTooBig),
            429 => Some(Self::TooManyRequests),
            900 => Some(Self::ConnectionFailed),
            901 => Some(Self::SendFailed),
            902 => Some(Self::GameServerTimeout),
            997 => Some(Self::InvalidGroupCommand),
            998 => Some(Self::InvalidQuestion),
            999 => Some(Self::InvalidCommand),
            _ => None,
        }
    }

    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::NameInUse => "this username is already taken",
            Self::NoContent => "there is nothing to show",
            Self::NoExit => "there is no exit that way",
            Self::BadRequest => "bad request",
            Self::NotInGroup => "you are not in a group",
            Self::AlreadyInGroup => "you are already in a group",
            Self::Forbidden => "you are not allowed to do that",
            Self::NotFound => "not found",
            Self::ActionNotAllowed => "action not allowed",
            Self::NoQuestAvailable => "no quest available",
            Self::NotInSameRoom => "not in this room",
            Self::NpcInCombat => "this npc is already fighting someone",
            Self::ActionAlreadyTaken => "you have already taken your action",
            Self::PlayerAlreadyInCombat => "you are already in a fight",
            Self::PlayerNotInCombat => "you are not in a fight",
            Self::FileNotFound => "the server could not load the requested file",
            Self::RoomNotFound => "room not found",
            Self::MissingItem => "an item is required to enter this room",
            Self::NotUsable => "item cannot be used",
            Self::DataTooBig => "sent data is too big",
            Self::TooManyRequests => "too many commands sent in a short time",
            Self::ConnectionFailed => "server unavailable",
            Self::SendFailed => "the server could not deliver your message",
            Self::GameServerTimeout => "the game server did not answer in time",
            Self::InvalidGroupCommand | Self::InvalidQuestion | Self::InvalidCommand => {
                "the server could not process this command"
            }
        }
    }
}

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

    pub fn kind(&self) -> Option<ErrorCode> {
        self.code.and_then(ErrorCode::from_code)
    }

    pub fn default_message_from_code(code_opt: Option<i32>) -> String {
        code_opt
            .and_then(ErrorCode::from_code)
            .map_or("unknown server error", ErrorCode::message)
            .to_string()
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
