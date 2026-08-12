use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct GroupJoinCommand {
    pub leader_name: String,
}

#[derive(Debug, Clone)]
pub struct GroupJoinResponse {
    pub group_id: String,
}

impl Command for GroupJoinCommand {
    type ResponseData = GroupJoinResponse;

    fn encode(&self) -> String {
        format!("GROUP JOIN {}", self.leader_name)
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

        Ok(GroupJoinResponse { group_id })
    }

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.code {
            Some(402) => Some("already in a group".to_string()),
            Some(403) => Some("no such user or not invited".to_string()),
            Some(404) => Some("group not found".to_string()),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        if args.trim().is_empty() {
            return None;
        }
        Some(Self {
            leader_name: args.trim().to_string(),
        })
    }
}
