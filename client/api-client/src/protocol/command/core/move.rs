use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

pub struct MoveCommand {
    pub direction: String,
}

#[derive(Debug)]
pub struct MoveResponse {
    pub room_id: String,
}

impl Command for MoveCommand {
    type ResponseData = MoveResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok(format!("MOVE {}", self.direction)),
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

                let room_id = match response.arguments[0].strip_prefix("room=") {
                    Some(id) => id.to_string(),
                    None => return Err(CommandError {
                        code: None,
                        message: "invalid arguments: missing 'room=' prefix".to_string(),
                    }),
                };

                Ok(MoveResponse {
                    room_id,
                })
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn refine_error(&self, server_info: &ServerInfo, error: &mut CommandError) {
        error.with_message(match (server_info.protocol_version, error.code) {
            (1, Some(405)) => Some(format!("No exit in direction: {}", self.direction)),
            _ => None,
        })
    }
}
