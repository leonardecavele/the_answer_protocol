use crate::client::ServerInfo;
use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct QuestsCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestListEntry {
    pub quest_id: String,
    pub status: String,
    pub progress: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestsResponse {
    pub quest_list: Vec<QuestListEntry>,
}

impl Command for QuestsCommand {
    type ResponseData = QuestsResponse;

    fn create_command(&self, server_info: &ServerInfo) -> Result<String, CommandError> {
        match server_info.protocol_version {
            1 => Ok("QUESTS".to_string()),
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
                let quest_list: Vec<QuestListEntry> = serde_json::from_str(response.arguments.join("").as_str())
                    .map_err(|e| CommandError::invalid_json_response(e))?;
                
                Ok(QuestsResponse {
                    quest_list,
                })
            }
            v => Err(CommandError::version_not_implemented(v)),
        }
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }

}
