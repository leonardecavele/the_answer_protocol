use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct FightCreateCommand {
    pub npc_id: String,
}

#[derive(Debug, Clone)]
pub struct FightCreateResponse;

impl Command for FightCreateCommand {
    type ResponseData = FightCreateResponse;

    fn encode(&self) -> String {
        format!("FIGHT CREATE {}", self.npc_id)
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

    fn from_str(args: &str) -> Option<Self> {
        Some(Self {
            npc_id: args.to_string(),
        })
    }
}
