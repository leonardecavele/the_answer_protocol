use crate::{constantes::LOOT, player::PlayerId};
use json::JsonValue;

pub type Questid = String;

#[derive(Clone)]
pub struct Loot {
    pub qty: u32,
    pub chance: f32,
    pub loot_type: LOOT,
}

impl Loot {
    pub fn to_json(&self) -> JsonValue {
        json::object! {
            "qty" => self.qty,
            "chance" => self.chance,
            "type" => self.loot_type.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Quest {
    name: String,
    description: String,
    loots: Vec<Loot>,
}

impl Quest {
    pub fn new(json: &JsonValue) -> Option<Self> {
        let name = json["name"].as_str()?;
        let description = json["description"].as_str()?;

        let mut loots = Vec::new();
        if json["loots"].is_object() {
            for (key, val) in json["loots"].entries() {
                let loot_type = LOOT::from_string(key)?;
                let qty = val["qty"].as_u32().unwrap_or(1);
                let chance = val["chance"].as_f64().unwrap_or(100.0) as f32;
                loots.push(Loot {
                    qty,
                    chance,
                    loot_type,
                });
            }
        }

        Some(Self {
            name: name.to_string(),
            description: description.to_string(),
            loots,
        })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_json_loots(&self) -> JsonValue {
        let vec: Vec<JsonValue> = self.loots.iter().map(|loot| loot.to_json()).collect();
        JsonValue::Array(vec)
    }

    pub fn get_loots(&self) -> &Vec<Loot> {
        &self.loots
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum QuestState {
    InProgress,
    Completed,
    Failed,
}

impl QuestState {
    pub fn to_str(&self) -> &str {
        match self {
            QuestState::InProgress => "in progress",
            QuestState::Completed => "completed",
            QuestState::Failed => "failed",
        }
    }
}

#[derive(Debug)]
pub struct QuestInstance {
    player: PlayerId,
    quest: Questid,
    state: QuestState,
}

impl QuestInstance {
    pub fn new(player: PlayerId, quest: Questid, state: QuestState) -> Self {
        Self {
            player,
            quest,
            state,
        }
    }

    pub fn get_player(&self) -> PlayerId {
        self.player
    }

    pub fn get_quest_name(&self) -> Questid {
        self.quest.clone()
    }

    pub fn get_state(&self) -> QuestState {
        self.state.clone()
    }
}
