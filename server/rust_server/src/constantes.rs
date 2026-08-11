use std::time::Duration;

use crate::room::RoomId;

pub const NPC_DMG: u32 = 20;
pub const MIN_DMG_DEALT: u32 = 5;
pub const LOST_ITEM: u8 = 0;
pub const T_SHIRT: u8 = 2;
pub const LOST_ITEM_SPAWN: &str = "pature";
pub const LOST_ITEM_SPAWN_ID: RoomId = 2 as RoomId;
pub const PLAYER_ROOM_SPAWN: &str = "devant_l_ecole";
pub const MAX_TIME_FOR_COMBAT: Duration = Duration::from_secs(45);
pub const NPC_RESPAWN_TIME: Duration = Duration::from_secs(30);
pub const ITEM_DESPAWN_TIME: Duration = Duration::from_mins(1);
pub const TICK_TIME_AMPLIFICATION: u64 = 1;
pub const TICK_RATE: u16 = 10; // 48
pub const TICK_TIME: Duration =
    Duration::from_millis((1000 * TICK_TIME_AMPLIFICATION) / TICK_RATE as u64);
pub const BASE_COMMAND_RESPONSE: &str = "Duly noted.";
pub enum TickResult {
    TickEnd,
    Exit,
}

pub type Direction = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    NoError,
    NameInUse,
    NoExit,
    AlreadyConnected,
    InvalidScope,
    NotInGroup,
    NotGroupLeader,
    AlreadyInGroup,
    NoSuchUser,
    NotInvited,
    ItemNotFound,
    ItemNotInInventory,
    NpcNotFound,
    NpcNotInRoom,
    GroupNotFound,
    NoSuchGroup,
    NpcNotHostile,
    NoQuestAvailable,
    ConnectionFailed,
    SendFailed,
    InvalidCommand,
    InvalidQuestion,
    InvalidGroupCommand,
    PlayerNotFound,
    NpcInCombat,
    ActionAlreadyTaken,
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
            Self::NoSuchUser | Self::NotInvited | Self::NotGroupLeader => 403,
            Self::ItemNotFound
            | Self::ItemNotInInventory
            | Self::NpcNotFound
            | Self::GroupNotFound
            | Self::NoSuchGroup => 404,
            Self::PlayerNotFound => 405,
            Self::NpcNotHostile => 405,
            Self::NoQuestAvailable => 406,
            Self::NpcNotInRoom => 407,
            Self::NpcInCombat => 408,
            Self::ActionAlreadyTaken => 409,
            Self::ConnectionFailed => 900,
            Self::SendFailed => 901,
            Self::InvalidGroupCommand => 997,
            Self::InvalidQuestion => 998,
            Self::InvalidCommand => 999,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LOOT {
    XP,
    TShirt,
    Bottle,
}

impl LOOT {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "XP" => Some(Self::XP),
            "TSHIRT" => Some(Self::TShirt),
            "BOTTLE" => Some(Self::Bottle),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::XP => "XP".to_string(),
            Self::TShirt => "TSHIRT".to_string(),
            Self::Bottle => "BOTTLE".to_string(),
        }
    }
}

pub const NPC_QUEST_GIVER: u8 = 1 << 0;
pub const NPC_MOB: u8 = 1 << 1;
pub const NPC_TALKER: u8 = 1 << 2;

pub const NO_MORE_MESSAGES: &str = "[end of dialogue]";
pub const PLAYER_STARTING_MAX_HP: u32 = 100;
pub const PLAYER_STARTING_HP: u32 = PLAYER_STARTING_MAX_HP;
