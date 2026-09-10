use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UseCommand {
    pub item_identifier: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UseResponse {
    pub r#type: String,
    pub context: HashMap<String, String>,
}

impl Command for UseCommand {
    type ResponseData = UseResponse;

    fn encode(&self) -> String {
        format!("USE {}", self.item_identifier)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        let use_response: UseResponse = serde_json::from_str(response.arguments.join(" ").as_str())
            .map_err(CommandError::invalid_json_response)?;

        Ok(use_response)
    }

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.code {
            Some(404) => Some("item not found".to_string()),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        if args.trim().is_empty() {
            return None;
        }

        Some(Self {
            item_identifier: args.trim().to_string(),
        })
    }
}
