use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NpcType {
    Enemy,
    QuestGiver,
    Dialogue,
    Normal,
}

impl Default for NpcType {
    fn default() -> Self {
        NpcType::Normal
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NpcEntry {
    pub name: String,
    #[serde(default)]
    pub npc_type: NpcType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemEntry {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RoomEntry {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Manifest {
    #[serde(default)]
    pub npcs: HashMap<String, NpcEntry>,
    #[serde(default)]
    pub items: HashMap<String, ItemEntry>,
    #[serde(default)]
    pub rooms: HashMap<String, RoomEntry>,
}

impl Manifest {
    pub fn load() -> Result<Self, String> {
        let content = fs::read_to_string(crate::constants::ASSETS_PATH_MANIFEST)
            .map_err(|e| format!("Failed to read {}: {}", crate::constants::ASSETS_PATH_MANIFEST, e))?;

        let manifest = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in manifest.json: {}", e))?;

        Ok(manifest)
    }
}
