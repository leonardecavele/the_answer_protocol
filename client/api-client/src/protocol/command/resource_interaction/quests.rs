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

    fn create_command(&self) -> String {
        "QUESTS".to_string()
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        let quest_list: Vec<QuestListEntry> =
            serde_json::from_str(response.arguments.join(" ").as_str())
                .map_err(CommandError::invalid_json_response)?;

        Ok(QuestsResponse { quest_list })
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
