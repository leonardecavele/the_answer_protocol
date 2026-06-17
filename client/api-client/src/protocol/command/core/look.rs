use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;

pub struct LookCommand;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LookResponse {
    pub room: LookRoom,
    pub players: Vec<String>,
    pub items: Vec<String>,
    pub npcs: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LookRoom {
    pub id: String,
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, String>,
}

impl Command for LookCommand {
    type ResponseData = LookResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok("LOOK".to_string()),
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
                if response.arguments.len() < 2 {
                    return Err(CommandError {
                        code: None,
                        message: "invalid arguments".to_string(),
                    });
                }

                let look_response: LookResponse =
                    serde_json::from_str(response.arguments.join("").as_str())
                        .map_err(|e| CommandError::invalid_json_response(e))?;

                Ok(look_response)
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }
}
