use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

pub struct LookCommand;

pub struct LookResponse {
    pub json_data: String,
}

impl Command for LookCommand {
    type ResponseData = LookResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok("LOOK".to_string()),
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
                if response.arguments.len() < 2 {
                    return Err(CommandError {
                        code: None,
                        message: "invalid arguments".to_string(),
                    });
                }

                // TD: create HELPER to parse JSON data: arguments[1..].join(" ")

                Ok(LookResponse {
                    json_data: response.arguments[1..].join(" "),
                })
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }
}
