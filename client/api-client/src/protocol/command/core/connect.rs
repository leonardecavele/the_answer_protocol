use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ConnectCommand {
    pub player_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectResponse {
    pub player_name: String,
}

impl Command for ConnectCommand {
    type ResponseData = ConnectResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok(format!("CONNECT {}", self.player_name)),
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
                if response.arguments.len() != 1 || response.arguments[0] != "connected" {
                    return Err(CommandError {
                        code: None,
                        message: "invalid arguments".to_string(),
                    });
                }
                Ok(ConnectResponse {
                    player_name: self.player_name.clone(),
                })
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn refine_error(&self, server_info: &ServerInfo, error: &mut CommandError) {
        error.with_message(match (server_info.protocol_version, error.code) {
            (1, Some(201)) => Some(format!("{} already taken", self.player_name)),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        if args.trim().is_empty() { return None; }
        Some(Self {
            player_name: args.trim().to_string(),
        })
    }

}
