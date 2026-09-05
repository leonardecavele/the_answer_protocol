use runtime::Assets;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const MANIFEST_FILE: &str = "manifest.json";

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
    pub fn load(assets: &Assets) -> Result<Self, String> {
        let content = assets
            .read(MANIFEST_FILE)
            .ok_or_else(|| format!("Failed to read {}", MANIFEST_FILE))?;

        let manifest = serde_json::from_slice(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", MANIFEST_FILE, e))?;

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
