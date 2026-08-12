use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct InventoryCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryResponse {
    pub inventory: Vec<String>,
}

impl Command for InventoryCommand {
    type ResponseData = InventoryResponse;

    fn create_command(&self) -> String {
        "INVENTORY".to_string()
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        let inventory: Vec<String> = serde_json::from_str(response.arguments.join(" ").as_str())
            .map_err(CommandError::invalid_json_response)?;

        Ok(InventoryResponse { inventory })
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
