use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct GroupLeaveCommand;

#[derive(Debug, Clone)]
pub struct GroupLeaveResponse {}

impl Command for GroupLeaveCommand {
    type ResponseData = GroupLeaveResponse;

    fn create_command(&self) -> String {
        "GROUP LEAVE".to_string()
    }

    fn parse_response(
        &self,
        _response: ServerResponse,
    ) -> Result<Self::ResponseData, CommandError> {
        Ok(GroupLeaveResponse {})
    }

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.code {
            Some(401) => Some("not in a group".to_string()),
            Some(404) => Some("group not found".to_string()),
            _ => None,
        })
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
