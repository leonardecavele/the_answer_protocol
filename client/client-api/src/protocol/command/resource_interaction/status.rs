use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct StatusCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatus {
    pub hp: u32,
    pub max_hp: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub player_status: PlayerStatus,
}

impl Command for StatusCommand {
    type ResponseData = StatusResponse;

    fn encode(&self) -> String {
        "STATUS".to_string()
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        let player_status: PlayerStatus =
            serde_json::from_str(response.arguments.join(" ").as_str())
                .map_err(CommandError::invalid_json_response)?;

        Ok(StatusResponse { player_status })
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
