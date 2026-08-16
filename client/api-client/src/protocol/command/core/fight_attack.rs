use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

const SUCCESS_KEY: &str = "SUCCEED";
const FAILURE_KEY: &str = "FAIL";

#[derive(Debug, Clone)]
pub enum FightAttackStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone)]
pub struct FightAttackCommand {
    pub code: String,
}

#[derive(Debug, Clone)]
pub struct FightAttackResponse {
    pub status: FightAttackStatus,
}

impl Command for FightAttackCommand {
    type ResponseData = FightAttackResponse;

    fn encode(&self) -> String {
        format!("FIGHT ATTACK {}", self.code)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        let arguments_str = response.arguments.join(" ");
        let failed_arguments_message = format!(
            "invalid arguments. expected OK {}/{}, got: {}",
            SUCCESS_KEY, FAILURE_KEY, arguments_str
        );

        if response.arguments.len() != 1 {
            return Err(CommandError {
                code: None,
                message: failed_arguments_message,
            });
        }

        if ![SUCCESS_KEY, FAILURE_KEY].contains(&response.arguments[0].as_str()) {
            return Err(CommandError {
                code: None,
                message: failed_arguments_message,
            });
        }

        let status = if response.arguments[0].as_str() == SUCCESS_KEY {
            FightAttackStatus::Success
        } else {
            FightAttackStatus::Failure
        };

        Ok(FightAttackResponse { status })
    }

    fn from_str(args: &str) -> Option<Self> {
        Some(Self {
            code: args.trim().to_string(),
        })
    }
}
