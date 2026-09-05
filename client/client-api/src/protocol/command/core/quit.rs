use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct QuitCommand;

#[derive(Debug, Clone)]
pub struct QuitResponse;

impl Command for QuitCommand {
    type ResponseData = QuitResponse;

    fn encode(&self) -> String {
        "QUIT".to_string()
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        if response.arguments.is_empty() || response.arguments[0] != "bye" {
            return Err(CommandError {
                code: None,
                message: "invalid arguments".to_string(),
            });
        }

        Ok(QuitResponse)
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
