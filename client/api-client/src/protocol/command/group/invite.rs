use crate::client::ServerInfo;
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

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok(format!("GROUP INVITE {}", self.username)),
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn parse_response(
        &self,
        server_info: &ServerInfo,
        _response: ServerResponse,
    ) -> Result<Self::ResponseData, CommandError> {
        match server_info.protocol_version {
            1 => Ok(GroupInviteResponse {}),
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn refine_error(&self, server_info: &ServerInfo, error: &mut CommandError) {
        error.with_message(match (server_info.protocol_version, error.code) {
            (1, Some(401)) => Some("not in a group".to_string()),
            (1, Some(402)) => Some("user already in a group".to_string()),
            (1, Some(403)) => Some("no such user".to_string()),
            (1, Some(404)) => Some("group not found".to_string()),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        if args.trim().is_empty() { return None; }
        Some(Self {
            username: args.trim().to_string(),
        })
    }

}
