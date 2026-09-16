use crate::error::{CommandError, ErrorCode};
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct GroupLeaveCommand;

#[derive(Debug, Clone)]
pub struct GroupLeaveResponse {}

impl Command for GroupLeaveCommand {
    type ResponseData = GroupLeaveResponse;

    fn encode(&self) -> String {
        "GROUP LEAVE".to_string()
    }

    fn parse_response(
        &self,
        _response: ServerResponse,
    ) -> Result<Self::ResponseData, CommandError> {
        Ok(GroupLeaveResponse {})
    }

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.kind() {
            Some(ErrorCode::NotFound) => Some("group not found".to_string()),
            _ => None,
        })
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
