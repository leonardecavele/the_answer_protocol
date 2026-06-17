use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

pub struct GroupJoinCommand {
    pub leader_name: String,
}

#[derive(Debug, Clone)]
pub struct GroupJoinResponse {
    pub group_id: String,
}

impl Command for GroupJoinCommand {
    type ResponseData = GroupJoinResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok(format!("GROUP JOIN {}", self.leader_name)),
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
                    None => {
                        return Err(CommandError {
                            code: None,
                            message: "invalid arguments: missing 'group=' prefix".to_string(),
                        })
                    }
                };

                Ok(GroupJoinResponse { group_id })
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn refine_error(&self, server_info: &ServerInfo, error: &mut CommandError) {
        error.with_message(match (server_info.protocol_version, error.code) {
            (1, Some(402)) => Some("already in a group".to_string()),
            (1, Some(403)) => {
                // The error code 403 can mean NO_SUCH_USER or NOT_INVITED
                // For simplicity, we just keep the default message or customize if possible
                // TapError default is usually enough, but we can override it
                Some("no such user or not invited".to_string())
            }
            (1, Some(404)) => Some("group not found".to_string()),
            _ => None,
        })
    }
}
