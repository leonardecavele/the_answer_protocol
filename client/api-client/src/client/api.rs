use crate::client::Client;
use crate::error::{CommandError, TapError};
use crate::protocol::command::communication::global_chat::{GlobalChatCommand, GlobalChatResponse};
use crate::protocol::command::communication::private_chat::{
    PrivateChatCommand, PrivateChatResponse,
};
use crate::protocol::command::core::connect::{ConnectCommand, ConnectResponse};
use crate::protocol::command::core::look::{LookCommand, LookResponse};
use crate::protocol::command::core::quit::QuitCommand;
use crate::protocol::command::core::who::{WhoCommand, WhoResponse};
use crate::protocol::command::group::create::{GroupCreateCommand, GroupCreateResponse};
use crate::protocol::command::group::invite::{GroupInviteCommand, GroupInviteResponse};
use crate::protocol::command::group::join::{GroupJoinCommand, GroupJoinResponse};
use crate::protocol::command::group::leave::{GroupLeaveCommand, GroupLeaveResponse};
use crate::protocol::command::resource_interaction::attack::{AttackCommand, AttackResponse};
use crate::protocol::command::resource_interaction::drop::{DropCommand, DropResponse};
use crate::protocol::command::resource_interaction::inventory::{InventoryCommand, InventoryResponse};
use crate::protocol::command::resource_interaction::quest::{QuestCommand, QuestResponse};
use crate::protocol::command::resource_interaction::quests::{QuestsCommand, QuestsResponse};
use crate::protocol::command::resource_interaction::status::{StatusCommand, StatusResponse};
use crate::protocol::command::resource_interaction::take::{TakeCommand, TakeResponse};
use crate::protocol::command::resource_interaction::talk::{TalkCommand, TalkResponse};
use tracing::debug;

impl Client {
    pub async fn connect(
        &mut self,
        player_name: String,
    ) -> Result<Result<ConnectResponse, CommandError>, TapError> {
        debug!("sending connect request for player: {}", player_name);

        let response = self.request(ConnectCommand { player_name }).await?;

        if let Ok(result) = &response {
            self.game.player_name = Some(result.player_name.to_string());
        }

        Ok(response)
    }

    pub async fn look(&mut self) -> Result<Result<LookResponse, CommandError>, TapError> {
        debug!("sending look request");

        let response = self.request(LookCommand).await?;

        if let Ok(result) = &response {
            self.game.world = Some(result.clone());
        }

        Ok(response)
    }

    pub async fn chat_global(
        &self,
        message: String,
    ) -> Result<Result<GlobalChatResponse, CommandError>, TapError> {
        debug!("sending look request");

        let response = self.request(GlobalChatCommand { message }).await?;

        Ok(response)
    }

    pub async fn chat_private(
        &self,
        to: String,
        message: String,
    ) -> Result<Result<PrivateChatResponse, CommandError>, TapError> {
        debug!("sending look request");

        let response = self.request(PrivateChatCommand { to, message }).await?;

        Ok(response)
    }

    pub async fn who(&self) -> Result<Result<WhoResponse, CommandError>, TapError> {
        debug!("sending who request");

        let response = self.request(WhoCommand).await?;

        Ok(response)
    }

    pub async fn group_create(&mut self) -> Result<Result<GroupCreateResponse, CommandError>, TapError> {
        debug!("sending group create request");

        let response = self.request(GroupCreateCommand).await?;

        if let Ok(result) = &response {
            self.game.group_id = Some(result.group_id.to_string());
        }

        Ok(response)
    }

    pub async fn group_invite(&self, username: String) -> Result<Result<GroupInviteResponse, CommandError>, TapError> {
        debug!("sending group invite request");

        let response = self.request(GroupInviteCommand { username }).await?;

        Ok(response)
    }

    pub async fn group_join(&mut self, leader_name: String) -> Result<Result<GroupJoinResponse, CommandError>, TapError> {
        debug!("sending group join request");

        let response = self.request(GroupJoinCommand { leader_name }).await?;

        if let Ok(result) = &response {
            self.game.group_id = Some(result.group_id.to_string());
        }

        Ok(response)
    }

    pub async fn group_leave(&mut self) -> Result<Result<GroupLeaveResponse, CommandError>, TapError> {
        debug!("sending group leave request");

        let response = self.request(GroupLeaveCommand).await?;

        if response.is_ok() {
            self.game.group_id = None;
        }

        Ok(response)
    }

    pub async fn take(&self, item_identifier: String) -> Result<Result<TakeResponse, CommandError>, TapError> {
        debug!("sending take request");
        Ok(self.request(TakeCommand { item_identifier }).await?)
    }

    pub async fn drop_item(&self, item_identifier: String) -> Result<Result<DropResponse, CommandError>, TapError> {
        debug!("sending drop request");
        Ok(self.request(DropCommand { item_identifier }).await?)
    }

    pub async fn inventory(&self) -> Result<Result<InventoryResponse, CommandError>, TapError> {
        debug!("sending inventory request");
        Ok(self.request(InventoryCommand).await?)
    }

    pub async fn talk(&self, npc_name: String) -> Result<Result<TalkResponse, CommandError>, TapError> {
        debug!("sending talk request");
        Ok(self.request(TalkCommand { npc_name }).await?)
    }

    pub async fn attack(&self, npc_name: String) -> Result<Result<AttackResponse, CommandError>, TapError> {
        debug!("sending attack request");
        Ok(self.request(AttackCommand { npc_name }).await?)
    }

    pub async fn status(&self) -> Result<Result<StatusResponse, CommandError>, TapError> {
        debug!("sending status request");
        Ok(self.request(StatusCommand).await?)
    }

    pub async fn quest(&self, npc_name: String) -> Result<Result<QuestResponse, CommandError>, TapError> {
        debug!("sending quest request");
        Ok(self.request(QuestCommand { npc_name }).await?)
    }

    pub async fn quests(&self) -> Result<Result<QuestsResponse, CommandError>, TapError> {
        debug!("sending quests request");
        Ok(self.request(QuestsCommand).await?)
    }

    pub async fn quit(self) {
        debug!("sending quit request");

        let _ = self.request(QuitCommand).await;
        drop(self)
    }
}
