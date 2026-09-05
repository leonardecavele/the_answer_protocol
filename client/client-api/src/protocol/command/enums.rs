use crate::commands::{
    FightAttackCommand, FightAttackResponse, FightCreateCommand, FightCreateResponse,
};
use crate::error::CommandError;
use crate::protocol::request::RequestFlow;

// Command Imports
use crate::protocol::command::communication::global_chat::GlobalChatCommand;
use crate::protocol::command::communication::group_chat::GroupChatCommand;
use crate::protocol::command::communication::private_chat::PrivateChatCommand;
use crate::protocol::command::communication::room_chat::RoomChatCommand;
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
use crate::protocol::command::communication::group_chat::GroupChatResponse;
use crate::protocol::command::communication::private_chat::PrivateChatResponse;
use crate::protocol::command::communication::room_chat::RoomChatResponse;
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

fn strip_keyword<'a>(input: &'a str, keyword: &str) -> Option<&'a str> {
    let mut rest = input.trim_start();

    for word in keyword.split_whitespace() {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let (head, tail) = rest.split_at(end);

        if !head.eq_ignore_ascii_case(word) {
            return None;
        }

        rest = tail.trim_start();
    }

    Some(rest)
}

macro_rules! define_api_protocol {
    (
        $(
            $variant:ident($cmd_type:ty, $resp_type:ty) => [$($keyword:expr),+ $(,)?]
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
                $(
                    $(
                        if let Some(args) = strip_keyword(input, $keyword) {
                            return <$cmd_type>::from_str(args).map(ApiRequest::$variant);
                        }
                    )+
                )*

                None
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
    Connect(ConnectCommand, ConnectResponse) => ["connect"],
    Quit(QuitCommand, QuitResponse) => ["quit"],
    Look(LookCommand, LookResponse) => ["look"],
    Move(MoveCommand, MoveResponse) => ["move"],
    Who(WhoCommand, WhoResponse) => ["who"],
    FightCreate(FightCreateCommand, FightCreateResponse) => ["fight create", "fc"],
    FightAttack(FightAttackCommand, FightAttackResponse) => ["fight attack", "fa"],
    GlobalChat(GlobalChatCommand, GlobalChatResponse) => ["chat global", "say"],
    RoomChat(RoomChatCommand, RoomChatResponse) => ["chat room", "cr"],
    GroupChat(GroupChatCommand, GroupChatResponse) => ["chat group", "cg"],
    PrivateChat(PrivateChatCommand, PrivateChatResponse) => ["chat private", "msg"],
    Take(TakeCommand, TakeResponse) => ["take"],
    Drop(DropCommand, DropResponse) => ["drop"],
    Inventory(InventoryCommand, InventoryResponse) => ["inventory", "inv"],
    Status(StatusCommand, StatusResponse) => ["status"],
    Talk(TalkCommand, TalkResponse) => ["talk"],
    Attack(AttackCommand, AttackResponse) => ["attack"],
    Quest(QuestCommand, QuestResponse) => ["quest"],
    Quests(QuestsCommand, QuestsResponse) => ["quests"],
    GroupCreate(GroupCreateCommand, GroupCreateResponse) => ["group create", "gc"],
    GroupJoin(GroupJoinCommand, GroupJoinResponse) => ["group join", "gj"],
    GroupLeave(GroupLeaveCommand, GroupLeaveResponse) => ["group leave", "gl"],
    GroupInvite(GroupInviteCommand, GroupInviteResponse) => ["group invite", "gi"],
}

impl ApiRequest {
    pub(crate) fn flow(&self) -> RequestFlow {
        match self {
            ApiRequest::Quit(_) => RequestFlow::End,
            _ => RequestFlow::Continue,
        }
    }
}
