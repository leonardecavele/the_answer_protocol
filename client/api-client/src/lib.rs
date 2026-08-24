mod client;
mod error;
mod protocol;

pub use client::{Client, ClientConfig, Connection, ConnectionState, ServerInfo};
pub use error::{CommandError, InternalError, NetworkError, ProtocolError, TapError};
pub use protocol::command::{ApiRequest, ApiResponse, Command};

pub use protocol::frame::{Frame, FrameDirection};
pub use protocol::response::{Opcode, ServerResponse};

pub mod events {
    pub use crate::client::event::{
        ChatMessage, DeathData, FightResultData, FightStartData, GameServerEvent, GroupEvent,
        KillData, RoomEvent, ServerEvent, SpawnData,
    };
}

pub mod commands {
    pub use crate::protocol::command::communication::global_chat::{
        GlobalChatCommand, GlobalChatResponse,
    };
    pub use crate::protocol::command::communication::private_chat::{
        PrivateChatCommand, PrivateChatResponse,
    };
    pub use crate::protocol::command::core::connect::{ConnectCommand, ConnectResponse};
    pub use crate::protocol::command::core::fight_attack::{
        FightAttackCommand, FightAttackResponse,
    };
    pub use crate::protocol::command::core::fight_create::{
        FightCreateCommand, FightCreateResponse,
    };
    pub use crate::protocol::command::core::look::{LookCommand, LookResponse, LookRoom};
    pub use crate::protocol::command::core::r#move::{MoveCommand, MoveResponse};
    pub use crate::protocol::command::core::quit::{QuitCommand, QuitResponse};
    pub use crate::protocol::command::core::who::{WhoCommand, WhoResponse};
    pub use crate::protocol::command::group::create::{GroupCreateCommand, GroupCreateResponse};
    pub use crate::protocol::command::group::invite::{GroupInviteCommand, GroupInviteResponse};
    pub use crate::protocol::command::group::join::{GroupJoinCommand, GroupJoinResponse};
    pub use crate::protocol::command::group::leave::{GroupLeaveCommand, GroupLeaveResponse};
    pub use crate::protocol::command::resource_interaction::attack::{
        AttackCommand, AttackResponse, CombatResult,
    };
    pub use crate::protocol::command::resource_interaction::drop::{DropCommand, DropResponse};
    pub use crate::protocol::command::resource_interaction::inventory::{
        InventoryCommand, InventoryResponse,
    };
    pub use crate::protocol::command::resource_interaction::quest::{
        QuestCommand, QuestData, QuestResponse, QuestReward, QuestStatus, QuestsCommand,
        QuestsResponse,
    };
    pub use crate::protocol::command::resource_interaction::status::{
        PlayerStatus, StatusCommand, StatusResponse,
    };
    pub use crate::protocol::command::resource_interaction::take::{TakeCommand, TakeResponse};
    pub use crate::protocol::command::resource_interaction::talk::{TalkCommand, TalkResponse};
}
