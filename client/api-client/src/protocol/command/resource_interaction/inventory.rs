use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};

pub struct InventoryCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryResponse {
    pub inventory: Vec<String>,
}

impl Command for InventoryCommand {
    type ResponseData = InventoryResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok("INVENTORY".to_string()),
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
                let inventory: Vec<String> = serde_json::from_str(response.arguments.join("").as_str())
                    .map_err(|e| CommandError::invalid_json_response(e))?;
                
                Ok(InventoryResponse {
                    inventory,
                })
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }
}
