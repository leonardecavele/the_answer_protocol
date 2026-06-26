use crate::client::ServerInfo;
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

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok("WHO".to_string()),
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
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
