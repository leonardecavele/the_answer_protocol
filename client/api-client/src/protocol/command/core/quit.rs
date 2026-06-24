use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct QuitCommand;

#[derive(Debug, Clone)]
pub struct QuitResponse;

impl Command for QuitCommand {
    type ResponseData = QuitResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok("QUIT".to_string()),
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
                if response.arguments.len() < 1 || response.arguments[0] != "bye" {
                    return Err(CommandError {
                        code: None,
                        message: "invalid arguments".to_string(),
                    });
                }

                Ok(QuitResponse)
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }

}
