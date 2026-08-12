use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct TalkCommand {
    pub npc_name: String,
}

#[derive(Debug, Clone)]
pub struct TalkResponse {
    pub dialogue: String,
}

impl Command for TalkCommand {
    type ResponseData = TalkResponse;

    fn create_command(&self) -> String {
        format!("TALK {}", self.npc_name)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        if response.arguments.is_empty() {
            return Err(CommandError {
                code: None,
                message: "invalid arguments".to_string(),
            });
        }

        Ok(TalkResponse {
            dialogue: response.arguments.join(" "),
        })
    }

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.code {
            Some(404) => Some("npc not found".to_string()),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        if args.trim().is_empty() {
            return None;
        }
        Some(Self {
            npc_name: args.trim().to_string(),
        })
    }
}
