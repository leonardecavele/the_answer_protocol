use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct FightCreateCommand;

#[derive(Debug, Clone)]
pub struct FightCreateResponse;

impl Command for FightCreateCommand {
    type ResponseData = FightCreateResponse;

    fn encode(&self) -> String {
        "FIGHT CREATE".to_string()
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        let arguments_str = response.arguments.join(" ");
        let failed_arguments_message = format!(
            "invalid arguments. expected OK FIGHT CREATED, got: {}",
            arguments_str
        );

        if response.arguments.len() != 2 {
            return Err(CommandError {
                code: None,
                message: failed_arguments_message,
            });
        }

        if arguments_str != "FIGHT CREATED" {
            return Err(CommandError {
                code: None,
                message: failed_arguments_message,
            });
        }

        Ok(FightCreateResponse)
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
