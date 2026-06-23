use crate::error::CommandError;

// Command Imports
use crate::protocol::command::communication::global_chat::GlobalChatCommand;
use crate::protocol::command::communication::private_chat::PrivateChatCommand;
use crate::protocol::command::core::connect::ConnectCommand;
use crate::protocol::command::core::look::LookCommand;
use crate::protocol::command::core::quit::QuitCommand;
use crate::protocol::command::core::r#move::MoveCommand;
use crate::protocol::command::core::who::WhoCommand;
use crate::protocol::command::group::create::GroupCreateCommand;
use crate::protocol::command::group::invite::GroupInviteCommand;
use crate::protocol::command::group::join::GroupJoinCommand;
use crate::protocol::command::group::leave::GroupLeaveCommand;
use crate::protocol::command::resource_interaction::attack::AttackCommand;
use crate::protocol::command::resource_interaction::drop::DropCommand;
use crate::protocol::command::resource_interaction::inventory::InventoryCommand;
use crate::protocol::command::resource_interaction::quest::QuestCommand;
use crate::protocol::command::resource_interaction::quests::QuestsCommand;
use crate::protocol::command::resource_interaction::status::StatusCommand;
use crate::protocol::command::resource_interaction::take::TakeCommand;
use crate::protocol::command::resource_interaction::talk::TalkCommand;

// Response Imports
use crate::protocol::command::communication::global_chat::GlobalChatResponse;
use crate::protocol::command::communication::private_chat::PrivateChatResponse;
use crate::protocol::command::core::connect::ConnectResponse;
use crate::protocol::command::core::look::LookResponse;
use crate::protocol::command::core::quit::QuitResponse;
use crate::protocol::command::core::r#move::MoveResponse;
use crate::protocol::command::core::who::WhoResponse;
use crate::protocol::command::group::create::GroupCreateResponse;
use crate::protocol::command::group::invite::GroupInviteResponse;
use crate::protocol::command::group::join::GroupJoinResponse;
use crate::protocol::command::group::leave::GroupLeaveResponse;
use crate::protocol::command::resource_interaction::attack::AttackResponse;
use crate::protocol::command::resource_interaction::drop::DropResponse;
use crate::protocol::command::resource_interaction::inventory::InventoryResponse;
use crate::protocol::command::resource_interaction::quest::QuestResponse;
use crate::protocol::command::resource_interaction::quests::QuestsResponse;
use crate::protocol::command::resource_interaction::status::StatusResponse;
use crate::protocol::command::resource_interaction::take::TakeResponse;
use crate::protocol::command::resource_interaction::talk::TalkResponse;

pub enum ApiRequest {
    Connect(ConnectCommand),
    Quit(QuitCommand),
    Look(LookCommand),
    Move(MoveCommand),
    Who(WhoCommand),
    GlobalChat(GlobalChatCommand),
    PrivateChat(PrivateChatCommand),
    Take(TakeCommand),
    Drop(DropCommand),
    Inventory(InventoryCommand),
    Status(StatusCommand),
    Talk(TalkCommand),
    Attack(AttackCommand),
    Quest(QuestCommand),
    Quests(QuestsCommand),
    GroupCreate(GroupCreateCommand),
    GroupJoin(GroupJoinCommand),
    GroupLeave(GroupLeaveCommand),
    GroupInvite(GroupInviteCommand),
}

#[derive(Debug, Clone)]
pub enum ApiResponse {
    Connect(Result<ConnectResponse, CommandError>),
    Quit(Result<QuitResponse, CommandError>),
    Look(Result<LookResponse, CommandError>),
    Move(Result<MoveResponse, CommandError>),
    Who(Result<WhoResponse, CommandError>),
    GlobalChat(Result<GlobalChatResponse, CommandError>),
    PrivateChat(Result<PrivateChatResponse, CommandError>),
    Take(Result<TakeResponse, CommandError>),
    Drop(Result<DropResponse, CommandError>),
    Inventory(Result<InventoryResponse, CommandError>),
    Status(Result<StatusResponse, CommandError>),
    Talk(Result<TalkResponse, CommandError>),
    Attack(Result<AttackResponse, CommandError>),
    Quest(Result<QuestResponse, CommandError>),
    Quests(Result<QuestsResponse, CommandError>),
    GroupCreate(Result<GroupCreateResponse, CommandError>),
    GroupJoin(Result<GroupJoinResponse, CommandError>),
    GroupLeave(Result<GroupLeaveResponse, CommandError>),
    GroupInvite(Result<GroupInviteResponse, CommandError>),
}

impl ApiResponse {
    pub fn get_error(&self) -> Option<&CommandError> {
        match self {
            ApiResponse::Connect(Err(e)) => Some(e),
            ApiResponse::Quit(Err(e)) => Some(e),
            ApiResponse::Look(Err(e)) => Some(e),
            ApiResponse::Move(Err(e)) => Some(e),
            ApiResponse::Who(Err(e)) => Some(e),
            ApiResponse::GlobalChat(Err(e)) => Some(e),
            ApiResponse::PrivateChat(Err(e)) => Some(e),
            ApiResponse::Take(Err(e)) => Some(e),
            ApiResponse::Drop(Err(e)) => Some(e),
            ApiResponse::Inventory(Err(e)) => Some(e),
            ApiResponse::Status(Err(e)) => Some(e),
            ApiResponse::Talk(Err(e)) => Some(e),
            ApiResponse::Attack(Err(e)) => Some(e),
            ApiResponse::Quest(Err(e)) => Some(e),
            ApiResponse::Quests(Err(e)) => Some(e),
            ApiResponse::GroupCreate(Err(e)) => Some(e),
            ApiResponse::GroupJoin(Err(e)) => Some(e),
            ApiResponse::GroupLeave(Err(e)) => Some(e),
            ApiResponse::GroupInvite(Err(e)) => Some(e),
            _ => None,
        }
    }
}
