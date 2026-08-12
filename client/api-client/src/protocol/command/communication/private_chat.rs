use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PrivateChatCommand {
    pub to: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PrivateChatResponse;

impl Command for PrivateChatCommand {
    type ResponseData = PrivateChatResponse;

    fn encode(&self) -> String {
        format!("CHAT PRIVATE {} {}", self.to, self.message)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        if !response.arguments.is_empty() {
            return Err(CommandError {
                code: None,
                message: "invalid arguments".to_string(),
            });
        }
        Ok(PrivateChatResponse)
    }

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.code {
            Some(401) => Some("not in group".to_string()),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        let (to, message) = args.trim().split_once(' ')?;
        Some(Self {
            to: to.trim().to_string(),
            message: message.trim().to_string(),
        })
    }
}
