pub mod communication;
pub mod core;
pub mod enums;
pub mod group;
pub mod resource_interaction;

use crate::error::CommandError;
use crate::protocol::response::ServerResponse;
use serde_json::Error;

pub trait Command {
    type ResponseData;

    fn create_command(&self) -> String;

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError>;

    fn refine_error(&self, _error: &mut CommandError) {}

    fn from_str(args: &str) -> Option<Self>
    where
        Self: Sized,
    {
        let _ = args;
        None
    }
}

impl CommandError {
    pub fn from_response(response: ServerResponse) -> Self {
        if response.arguments.is_empty() {
            return CommandError {
                code: None,
                message: CommandError::default_message_from_code(None),
            };
        }

        let code = response.arguments[0].parse::<i32>().ok();
        let details = if response.arguments.len() > 1 {
            Some(response.arguments[1..].join(" "))
        } else {
            None
        };

        let friendly_message = CommandError::default_message_from_code(code);

        match (code, details) {
            (Some(code), details) => {
                let mut message = friendly_message;

                if let Some(details) = details {
                    message = format!("{}: {}", message, details);
                }

                CommandError {
                    code: Some(code),
                    message,
                }
            }
            (None, Some(details)) => CommandError {
                code: None,
                message: format!("{}: {}", friendly_message, details),
            },
            _ => CommandError {
                code: None,
                message: friendly_message,
            },
        }
    }

    pub fn invalid_json_response(e: Error) -> Self {
        CommandError {
            code: None,
            message: format!("invalid json response: {}", e),
        }
    }
}
