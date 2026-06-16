use crate::client::Client;
use crate::error::{CommandError, TapError};
use crate::protocol::command::connect::{ConnectCommand, ConnectResponse};
use crate::protocol::command::global_chat::{GlobalChatCommand, GlobalChatResponse};
use crate::protocol::command::look::{LookCommand, LookResponse};
use crate::protocol::command::quit::QuitCommand;
use tracing::debug;

impl Client {
    pub async fn connect(
        &self,
        player_name: String,
    ) -> Result<Result<ConnectResponse, CommandError>, TapError> {
        debug!("sending connect request for player: {}", player_name);

        let response = self.request(ConnectCommand { player_name }).await?;

        Ok(response)
    }

    pub async fn look(&self) -> Result<Result<LookResponse, CommandError>, TapError> {
        debug!("sending look request");

        let response = self.request(LookCommand).await?;

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

    pub async fn quit(self) {
        debug!("sending quit request");

        let _ = self.request(QuitCommand).await;
        drop(self)
    }
}
