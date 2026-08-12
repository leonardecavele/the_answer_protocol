use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct WhoCommand;

#[derive(Debug, Clone)]
pub struct WhoResponse {
    pub player_count: u32,
}

impl Command for WhoCommand {
    type ResponseData = WhoResponse;

    fn create_command(&self) -> String {
        "WHO".to_string()
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        if response.arguments.len() != 1 {
            return Err(CommandError {
                code: None,
                message: "invalid arguments".to_string(),
            });
        }

        let player_count = match response.arguments[0].strip_prefix("players=") {
            Some(count_str) => match count_str.parse::<u32>() {
                Ok(c) => c,
                Err(_) => {
                    return Err(CommandError {
                        code: None,
                        message: "invalid arguments: invalid number format".to_string(),
                    });
                }
            },
            None => {
                return Err(CommandError {
                    code: None,
                    message: "invalid arguments: missing 'players=' prefix".to_string(),
                });
            }
        };

        Ok(WhoResponse { player_count })
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
