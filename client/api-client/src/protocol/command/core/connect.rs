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

    fn encode(&self) -> String {
        format!("CONNECT {}", self.player_name)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
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

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.code {
            Some(201) => Some(format!("{} already taken", self.player_name)),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        if args.trim().is_empty() {
            return None;
        }
        Some(Self {
            player_name: args.trim().to_string(),
        })
    }
}
