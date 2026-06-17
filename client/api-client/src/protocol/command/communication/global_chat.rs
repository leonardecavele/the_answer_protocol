use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

pub struct GlobalChatCommand {
    pub message: String,
}

#[derive(Debug)]
pub struct GlobalChatResponse;

impl Command for GlobalChatCommand {
    type ResponseData = GlobalChatResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok(format!("CHAT GLOBAL {}", self.message)),
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
                if response.arguments.len() != 0 {
                    return Err(CommandError {
                        code: None,
                        message: "invalid arguments".to_string(),
                    });
                }
                Ok(GlobalChatResponse)
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn refine_error(&self, server_info: &ServerInfo, error: &mut CommandError) {
        error.with_message(match (server_info.protocol_version, error.code) {
            (1, Some(401)) => Some(format!("not in group")),
            _ => None,
        })
    }
}
