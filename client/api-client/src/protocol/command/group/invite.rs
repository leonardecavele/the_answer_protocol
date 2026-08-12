use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct GroupInviteCommand {
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct GroupInviteResponse {}

impl Command for GroupInviteCommand {
    type ResponseData = GroupInviteResponse;

    fn encode(&self) -> String {
        format!("GROUP INVITE {}", self.username)
    }

    fn parse_response(
        &self,
        _response: ServerResponse,
    ) -> Result<Self::ResponseData, CommandError> {
        Ok(GroupInviteResponse {})
    }

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.code {
            Some(401) => Some("not in a group".to_string()),
            Some(402) => Some("user already in a group".to_string()),
            Some(403) => Some("no such user".to_string()),
            Some(404) => Some("group not found".to_string()),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        if args.trim().is_empty() {
            return None;
        }
        Some(Self {
            username: args.trim().to_string(),
        })
    }
}
