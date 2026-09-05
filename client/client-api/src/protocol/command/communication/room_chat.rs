use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RoomChatCommand {
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RoomChatResponse;

impl Command for RoomChatCommand {
    type ResponseData = RoomChatResponse;

    fn encode(&self) -> String {
        format!("CHAT ROOM {}", self.message)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        if !response.arguments.is_empty() {
            return Err(CommandError {
                code: None,
                message: "invalid arguments".to_string(),
            });
        }
        Ok(RoomChatResponse)
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
