use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;

#[derive(Debug, Clone)]
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

    fn create_command(&self) -> String {
        "LOOK".to_string()
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        if response.arguments.len() < 2 {
            return Err(CommandError {
                code: None,
                message: "invalid arguments".to_string(),
            });
        }

        let look_response: LookResponse =
            serde_json::from_str(response.arguments.join(" ").as_str())
                .map_err(CommandError::invalid_json_response)?;

        Ok(look_response)
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
