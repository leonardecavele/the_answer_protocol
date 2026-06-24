use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct TakeCommand {
    pub item_identifier: String,
}

#[derive(Debug, Clone)]
pub struct TakeResponse {
    pub item_identifier: String,
}

impl Command for TakeCommand {
    type ResponseData = TakeResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok(format!("TAKE {}", self.item_identifier)),
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
                
                let item_identifier = match response.arguments[0].strip_prefix("taken=") {
                    Some(id) => id.to_string(),
                    None => return Err(CommandError {
                        code: None,
                        message: "invalid arguments: missing 'taken=' prefix".to_string(),
                    }),
                };
                
                Ok(TakeResponse {
                    item_identifier,
                })
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn refine_error(&self, server_info: &ServerInfo, error: &mut CommandError) {
        error.with_message(match (server_info.protocol_version, error.code) {
            (1, Some(404)) => Some("item not found".to_string()),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        if args.trim().is_empty() { return None; }
        Some(Self {
            item_identifier: args.trim().to_string(),
        })
    }

}
