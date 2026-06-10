use std::time::Duration;


pub const TICK_TIME_AMPLIFICATION: u64 = 1;
pub const TICK_RATE: u16 = 5; // 48
pub const TICK_TIME: Duration = Duration::from_millis((1000 * TICK_TIME_AMPLIFICATION) / TICK_RATE as u64);
pub const BASE_COMMAND_RESPONSE: &str = "Duly noted.";
pub enum TickResult {
    TickEnd,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    NoError,
    NameInUse,
    NoExit,
    AlreadyConnected,
    InvalidScope,
    NotInGroup,
    AlreadyInGroup,
    NoSuchUser,
    NotInvited,
    ItemNotFound,
    ItemNotInInventory,
    NpcNotFound,
    GroupNotFound,
    NoSuchGroup,
    NpcNotHostile,
    NoQuestAvailable,
    ConnectionFailed,
    SendFailed
    
}

impl ErrorCode {
    pub fn code(&self) -> u16 {
        match self {
            Self::NoError => 0,
            Self::NameInUse => 201,
            Self::NoExit => 301,
            Self::AlreadyConnected | Self::InvalidScope => 400,
            Self::NotInGroup => 401,
            Self::AlreadyInGroup => 402,
            Self::NoSuchUser | Self::NotInvited => 403,
            Self::ItemNotFound 
            | Self::ItemNotInInventory 
            | Self::NpcNotFound 
            | Self::GroupNotFound 
            | Self::NoSuchGroup => 404,
            Self::NpcNotHostile => 405,
            Self::NoQuestAvailable => 406,
            Self::ConnectionFailed => 900,
            Self::SendFailed => 901,
        }
    }
}