use crate::error::{CommandError, ErrorCode};
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AttackCommand {
    pub npc_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatResult {
    pub attacker_hp: u16,
    pub target_hp: u16,
    pub damage: u16,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackResponse {
    pub combat_result: CombatResult,
}

impl Command for AttackCommand {
    type ResponseData = AttackResponse;

    fn encode(&self) -> String {
        format!("ATTACK {}", self.npc_name)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        let combat_result: CombatResult =
            serde_json::from_str(response.arguments.join(" ").as_str())
                .map_err(CommandError::invalid_json_response)?;

        Ok(AttackResponse { combat_result })
    }

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.kind() {
            Some(ErrorCode::NotFound) => Some("npc not found".to_string()),
            Some(ErrorCode::ActionNotAllowed) => Some("this npc cannot be attacked".to_string()),
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
