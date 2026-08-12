use crate::error::CommandError;

// Command Imports
use crate::protocol::command::communication::global_chat::GlobalChatCommand;
use crate::protocol::command::communication::private_chat::PrivateChatCommand;
use crate::protocol::command::core::connect::ConnectCommand;
use crate::protocol::command::core::look::LookCommand;
use crate::protocol::command::core::r#move::MoveCommand;
use crate::protocol::command::core::quit::QuitCommand;
use crate::protocol::command::core::who::WhoCommand;
use crate::protocol::command::group::create::GroupCreateCommand;
use crate::protocol::command::group::invite::GroupInviteCommand;
use crate::protocol::command::group::join::GroupJoinCommand;
use crate::protocol::command::group::leave::GroupLeaveCommand;
use crate::protocol::command::resource_interaction::attack::AttackCommand;
use crate::protocol::command::resource_interaction::drop::DropCommand;
use crate::protocol::command::resource_interaction::inventory::InventoryCommand;
use crate::protocol::command::resource_interaction::quest::QuestCommand;
use crate::protocol::command::resource_interaction::quest::QuestsCommand;
use crate::protocol::command::resource_interaction::status::StatusCommand;
use crate::protocol::command::resource_interaction::take::TakeCommand;
use crate::protocol::command::resource_interaction::talk::TalkCommand;

// Response Imports
use crate::protocol::command::Command;
use crate::protocol::command::communication::global_chat::GlobalChatResponse;
use crate::protocol::command::communication::private_chat::PrivateChatResponse;
use crate::protocol::command::core::connect::ConnectResponse;
use crate::protocol::command::core::look::LookResponse;
use crate::protocol::command::core::r#move::MoveResponse;
use crate::protocol::command::core::quit::QuitResponse;
use crate::protocol::command::core::who::WhoResponse;
use crate::protocol::command::group::create::GroupCreateResponse;
use crate::protocol::command::group::invite::GroupInviteResponse;
use crate::protocol::command::group::join::GroupJoinResponse;
use crate::protocol::command::group::leave::GroupLeaveResponse;
use crate::protocol::command::resource_interaction::attack::AttackResponse;
use crate::protocol::command::resource_interaction::drop::DropResponse;
use crate::protocol::command::resource_interaction::inventory::InventoryResponse;
use crate::protocol::command::resource_interaction::quest::QuestResponse;
use crate::protocol::command::resource_interaction::quest::QuestsResponse;
use crate::protocol::command::resource_interaction::status::StatusResponse;
use crate::protocol::command::resource_interaction::take::TakeResponse;
use crate::protocol::command::resource_interaction::talk::TalkResponse;

macro_rules! define_api_protocol {
    (
        $(
            $variant:ident($cmd_type:ty, $resp_type:ty) => $cmd_name:expr
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone)]
        pub enum ApiRequest {
            $(
                $variant($cmd_type),
            )*
        }

        impl ApiRequest {
            pub fn parse(input: &str) -> Option<Self> {
                let mut parts = input.trim().splitn(2, ' ');
                let keyword = parts.next()?.to_lowercase();
                let args = parts.next().unwrap_or("");

                match keyword.as_str() {
                    $(
                        $cmd_name => <$cmd_type>::from_str(args).map(ApiRequest::$variant),
                    )*
                    _ => None,
                }
            }
        }

        #[derive(Debug, Clone)]
        pub enum ApiResponse {
            $(
                $variant(Result<$resp_type, CommandError>),
            )*
        }

        impl ApiResponse {
            pub fn get_error(&self) -> Option<&CommandError> {
                match self {
                    $(
                        ApiResponse::$variant(Err(e)) => Some(e),
                    )*
                    _ => None,
                }
            }
        }
    };
}

define_api_protocol! {
    Connect(ConnectCommand, ConnectResponse) => "connect",
    Quit(QuitCommand, QuitResponse) => "quit",
    Look(LookCommand, LookResponse) => "look",
    Move(MoveCommand, MoveResponse) => "move",
    Who(WhoCommand, WhoResponse) => "who",
    GlobalChat(GlobalChatCommand, GlobalChatResponse) => "say",
    PrivateChat(PrivateChatCommand, PrivateChatResponse) => "msg",
    Take(TakeCommand, TakeResponse) => "take",
    Drop(DropCommand, DropResponse) => "drop",
    Inventory(InventoryCommand, InventoryResponse) => "inv",
    Status(StatusCommand, StatusResponse) => "status",
    Talk(TalkCommand, TalkResponse) => "talk",
    Attack(AttackCommand, AttackResponse) => "attack",
    Quest(QuestCommand, QuestResponse) => "quest",
    Quests(QuestsCommand, QuestsResponse) => "quests",
    GroupCreate(GroupCreateCommand, GroupCreateResponse) => "gc",
    GroupJoin(GroupJoinCommand, GroupJoinResponse) => "gj",
    GroupLeave(GroupLeaveCommand, GroupLeaveResponse) => "gl",
    GroupInvite(GroupInviteCommand, GroupInviteResponse) => "gi",
}
