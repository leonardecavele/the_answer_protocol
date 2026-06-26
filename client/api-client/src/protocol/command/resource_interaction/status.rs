use crate::client::ServerInfo;
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

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok("STATUS".to_string()),
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn parse_response(
        &self,
        server_info: &ServerInfo,
        response: ServerResponse,
    ) -> Result<Self::ResponseData, CommandError> {
        match server_info.protocol_version {
            1 => {
                let player_status: PlayerStatus =
                    serde_json::from_str(response.arguments.join(" ").as_str())
                        .map_err(|e| CommandError::invalid_json_response(e))?;

                Ok(StatusResponse { player_status })
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
