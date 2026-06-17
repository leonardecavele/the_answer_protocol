use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};

pub struct AttackCommand {
    pub npc_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatResult {
    pub attacker_hp: u32,
    pub target_hp: u32,
    pub damage: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackResponse {
    pub combat_result: CombatResult,
}

impl Command for AttackCommand {
    type ResponseData = AttackResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok(format!("ATTACK {}", self.npc_name)),
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn parse_response(
        &self,
        server_info: &ServerInfo,
        response: ServerResponse,
    ) -> Result<Self::ResponseData, CommandError> {
        match server_info.protocol_version {
            1 => {
                let combat_result: CombatResult = serde_json::from_str(response.arguments.join("").as_str())
                    .map_err(|e| CommandError::invalid_json_response(e))?;
                
                Ok(AttackResponse {
                    combat_result,
                })
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn refine_error(&self, server_info: &ServerInfo, error: &mut CommandError) {
        error.with_message(match (server_info.protocol_version, error.code) {
            (1, Some(400)) => Some("npc not hostile".to_string()),
            (1, Some(404)) => Some("npc not found".to_string()),
            _ => None,
        })
    }
}
