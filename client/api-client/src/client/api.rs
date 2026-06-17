use crate::client::Client;
use crate::error::{CommandError, TapError};
use crate::protocol::command::communication::global_chat::{GlobalChatCommand, GlobalChatResponse};
use crate::protocol::command::communication::private_chat::{
    PrivateChatCommand, PrivateChatResponse,
};
use crate::protocol::command::core::connect::{ConnectCommand, ConnectResponse};
use crate::protocol::command::core::look::{LookCommand, LookResponse};
use crate::protocol::command::core::quit::QuitCommand;
use crate::protocol::command::group::create::{GroupCreateCommand, GroupCreateResponse};
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

    pub async fn group_create(
        &mut self,
    ) -> Result<Result<GroupCreateResponse, CommandError>, TapError> {
        debug!("sending group create request");

        let response = self.request(GroupCreateCommand).await?;

        if let Ok(result) = &response {
            self.game.group_id = Some(result.group_id.to_string());
        }

        Ok(response)
    }

    pub async fn quit(self) {
        debug!("sending quit request");

        let _ = self.request(QuitCommand).await;
        drop(self)
    }
}
