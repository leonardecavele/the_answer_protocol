use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GlobalChatCommand {
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalChatResponse;

impl Command for GlobalChatCommand {
    type ResponseData = GlobalChatResponse;

    fn create_command(&self) -> String {
        format!("CHAT GLOBAL {}", self.message)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        if !response.arguments.is_empty() {
            return Err(CommandError {
                code: None,
                message: "invalid arguments".to_string(),
            });
        }
        Ok(GlobalChatResponse)
    }

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.code {
            Some(401) => Some("not in group".to_string()),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        if args.trim().is_empty() {
            return None;
        }
        Some(Self {
            message: args.trim().to_string(),
        })
    }
}
