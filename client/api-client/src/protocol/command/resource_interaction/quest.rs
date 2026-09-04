use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::warn;

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub enum QuestStatus {
    InProgress,
    Completed,
}

impl<'de> Deserialize<'de> for QuestStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?.replace(" ", "");

        let status = if value.eq_ignore_ascii_case("inprogress") {
            QuestStatus::InProgress
        } else if value.eq_ignore_ascii_case("completed") {
            QuestStatus::Completed
        } else {
            warn!("Unknown quest status: {}", value);
            QuestStatus::InProgress
        };

        Ok(status)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestReward {
    pub qty: u32,
    pub chance: u32,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestData {
    pub name: String,
    pub description: String,
    pub reward: Vec<QuestReward>,
    pub status: QuestStatus,
}

impl QuestData {
    pub fn is_completed(&self) -> bool {
        self.status == QuestStatus::Completed
    }
}

// =============================
// ========= QUEST =============
// =============================

#[derive(Debug, Clone)]
pub struct QuestCommand {
    pub npc_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestResponse {
    pub quest_data: QuestData,
}

impl Command for QuestCommand {
    type ResponseData = QuestResponse;

    fn encode(&self) -> String {
        format!("QUEST {}", self.npc_name)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        let quest_data: QuestData = serde_json::from_str(response.arguments.join(" ").as_str())
            .map_err(CommandError::invalid_json_response)?;

        Ok(QuestResponse { quest_data })
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

// ==============================
// ========= QUESTS =============
// ==============================

#[derive(Debug, Clone)]
pub struct QuestsCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestsResponse {
    pub quest_list: Vec<QuestData>,
}

impl Command for QuestsCommand {
    type ResponseData = QuestsResponse;

    fn encode(&self) -> String {
        "QUESTS".to_string()
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        let quest_list: Vec<QuestData> =
            serde_json::from_str(response.arguments.join(" ").as_str())
                .map_err(CommandError::invalid_json_response)?;

        Ok(QuestsResponse { quest_list })
    }

    fn from_str(_args: &str) -> Option<Self> {
        Some(Self)
    }
}
