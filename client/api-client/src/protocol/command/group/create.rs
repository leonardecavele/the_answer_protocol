use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct GroupCreateCommand;

#[derive(Debug, Clone)]
pub struct GroupCreateResponse {
    pub group_id: String,
}

impl Command for GroupCreateCommand {
    type ResponseData = GroupCreateResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok("GROUP CREATE".to_string()),
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
                
                let group_id = match response.arguments[0].strip_prefix("group=") {
                    Some(id) => id.to_string(),
                    None => return Err(CommandError {
                        code: None,
                        message: "invalid arguments: missing 'group=' prefix".to_string(),
                    }),
                };
                
                Ok(GroupCreateResponse {
                    group_id,
                })
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn refine_error(&self, server_info: &ServerInfo, error: &mut CommandError) {
        error.with_message(match (server_info.protocol_version, error.code) {
            (1, Some(402)) => Some("already in a group".to_string()),
            _ => None,
        })
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }

}
