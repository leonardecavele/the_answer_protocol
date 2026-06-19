use json::JsonValue;
use tracing::error;

use crate::constantes::{NPC_MOB, NPC_QUEST_GIVER, NPC_TALKER};
use crate::room::RoomName;

type NpcType = u8;
pub type NpcId = u32;

// for now, use this. later create Dialog and Questid structs
pub type Dialog = String;
pub type Questid = u32;

#[derive(Clone)]
pub struct Npc {
    id: NpcId,
    name: String,
    npc_type: NpcType,
    hp: Option<u32>,
    max_hp: Option<u32>,
    dialogs: Option<Vec<Dialog>>,
    quests: Option<Vec<Questid>>,
    room_spawn: RoomName,
}

impl Npc {
    pub fn new(json: &JsonValue) -> Option<Self> {
        let id = json["id"].as_u32()?;
        let name = json["name"].as_str()?.to_string();
        let npc_type = json["npc_type"].as_u8()?;
        if npc_type < 1 || npc_type > NPC_QUEST_GIVER + NPC_MOB + NPC_TALKER {
            error!("invalid npc type: {}", npc_type);
            return None;
        }
        let hp = json["hp"].as_u32();
        let max_hp = json["max_hp"].as_u32();

        let dialogs: Option<Vec<Dialog>> =
            if json["dialogs"].is_array() && !json["dialogs"].is_empty() {
                let mut dialogs = Vec::new();
                for item in json["dialogs"].members() {
                    if let Some(dialog) = item.as_str() {
                        dialogs.push(dialog.to_string());
                    }
                }
                Some(dialogs)
            } else {
                None
            };

        let quests = if json["quests"].is_array() && !json["quests"].is_empty() {
            let mut quests = Vec::new();
            for item in json["quests"].members() {
                if let Some(id) = item.as_u32() {
                    quests.push(id);
                }
            }
            Some(quests)
        } else {
            None
        };

        let room_spawn = json["spawns"].as_str()?.to_string();

        Some(Self {
            id,
            name,
            npc_type,
            hp,
            max_hp,
            dialogs,
            quests,
            room_spawn,
        })
    }

    pub fn get_id(&self) -> NpcId {
        self.id
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_npc_type(&self) -> NpcType {
        self.npc_type
    }
    pub fn get_hp(&self) -> Option<u32> {
        self.hp
    }
    pub fn get_max_hp(&self) -> Option<u32> {
        self.max_hp
    }
    pub fn get_dialog(&self, index: usize) -> Option<&Dialog> {
        self.dialogs.as_ref()?.get(index)
    }
    pub fn get_spawn_room(&self) -> &str {
        &self.room_spawn
    }
}
