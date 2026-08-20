use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

pub const ASSETS_PATH_MANIFEST: &str = "../assets/manifest.json";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum NpcKind {
    Enemy,
    QuestGiver,
    Dialogue,
    #[default]
    Normal,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NpcEntry {
    pub name: String,
    #[serde(rename = "type", default)]
    pub kind: NpcKind,
    pub image_path: Option<String>,
    pub image_paths: Option<Vec<String>>,
    pub frame_ms: Option<u64>,
    #[serde(default)]
    pub actions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemEntry {
    pub name: String,
    pub description: String,
    pub image_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RoomEntry {
    pub image_path: Option<String>,
    pub direction: Option<char>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QuestConfig {
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
    #[serde(default)]
    pub quests: HashMap<String, QuestConfig>,
}

impl Manifest {
    pub fn load() -> Result<Self, String> {
        let content = fs::read_to_string(ASSETS_PATH_MANIFEST)
            .map_err(|e| format!("Failed to read {}: {}", ASSETS_PATH_MANIFEST, e))?;

        let manifest = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in manifest.json: {}", e))?;

        Ok(manifest)
    }

    pub fn npc_name(&self, id: &str) -> String {
        self.npcs
            .get(id)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| id.to_string())
    }

    pub fn item_name(&self, id: &str) -> String {
        self.items
            .get(id)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| id.to_string())
    }
}
