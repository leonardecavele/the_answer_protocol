use crate::constants::{PLAYER_ROOM_SPAWN, PLAYER_STARTING_HP, PLAYER_STARTING_MAX_HP};
use crate::inventory::Inventory;
use crate::quests::QuestState;
use crate::quests::Questid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Save {
    pub name: String,
    pub id: u32,
    pub hp: u32,
    pub max_hp: u32,
    #[serde(default)]
    pub inventory: Inventory,
    pub current_room: String,
    #[serde(default)]
    pub quests: Vec<(Questid, QuestState)>,
    #[serde(default)]
    pub completed_quests: HashMap<Questid, u32>,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            name: String::new(),
            id: 0,
            hp: PLAYER_STARTING_HP,
            max_hp: PLAYER_STARTING_MAX_HP,
            inventory: Inventory::new(),
            current_room: PLAYER_ROOM_SPAWN.to_string(),
            quests: Vec::new(),
            completed_quests: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerSave {
    pub next_player_id: u32,
    pub rooms_inventory: HashMap<String, Inventory>,
}

impl Default for ServerSave {
    fn default() -> Self {
        Self {
            next_player_id: 0,
            rooms_inventory: HashMap::new(),
        }
    }
}
