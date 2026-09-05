use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct FightAttackCommand {
    pub code: String,
}

#[derive(Debug, Clone)]
pub struct FightAttackResponse;

impl Command for FightAttackCommand {
    type ResponseData = FightAttackResponse;

    fn encode(&self) -> String {
        format!("FIGHT ATTACK {}", self.code)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        let arguments_str = response.arguments.join(" ");
        let failed_arguments_message = format!(
            "invalid arguments. expected OK Processing, got: {}",
            arguments_str
        );

        if response.arguments.len() != 1 || response.arguments[0] != "Processing" {
            return Err(CommandError {
                code: None,
                message: failed_arguments_message,
            });
        }

        Ok(FightAttackResponse)
    }

    fn from_str(args: &str) -> Option<Self> {
        Some(Self {
            code: args.trim().to_string(),
        })
    }
}
