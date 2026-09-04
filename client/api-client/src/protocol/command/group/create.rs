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

    fn encode(&self) -> String {
        "GROUP CREATE".to_string()
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
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
                });
            }
        };

        Ok(GroupCreateResponse { group_id })
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
