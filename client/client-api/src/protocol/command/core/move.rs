use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MoveCommand {
    pub direction: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MoveResponse {
    pub room_id: String,
}

impl Command for MoveCommand {
    type ResponseData = MoveResponse;

    fn encode(&self) -> String {
        format!("MOVE {}", self.direction)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        if response.arguments.len() != 1 {
            return Err(CommandError {
                code: None,
                message: "invalid arguments".to_string(),
            });
        }

        let room_id = match response.arguments[0].strip_prefix("room=") {
            Some(id) => id.to_string(),
            None => {
                return Err(CommandError {
                    code: None,
                    message: "invalid arguments: missing 'room=' prefix".to_string(),
                });
            }
        };

        Ok(MoveResponse { room_id })
    }

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.code {
            Some(301) => Some(format!("there is no exit to the {}", self.direction)),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        if args.trim().is_empty() {
            return None;
        }
        Some(Self {
            direction: args.trim().to_string(),
        })
    }
}
