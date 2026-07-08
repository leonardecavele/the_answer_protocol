use json::JsonValue;
use tracing::error;

use crate::constantes::{NPC_MOB, NPC_QUEST_GIVER, NPC_TALKER};
use crate::quests::Questid;
use crate::room::RoomName;

type NpcType = u8;
pub type NpcId = u32;

// for now, use this. later create Dialog and Questid structs
pub type Dialog = Vec<String>;

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
    pub fn new(json: &JsonValue, id: NpcId) -> Option<Self> {
        let name = json["name"].as_str()?.to_string();
        
        let max_hp = json["max_hp"].as_u32();
        let hp = max_hp;

        let dialogs: Option<Vec<Dialog>> =
            if json["dialogs"].is_array() && !json["dialogs"].is_empty() {
                let mut dialogs = Vec::new();
                for item in json["dialogs"].members() {
                    if item.is_array() {
                        let mut lines = Vec::new();
                        for line in item.members() {
                            if let Some(line_str) = line.as_str() {
                                lines.push(line_str.to_string());
                            }
                        }
                        dialogs.push(lines);
                    } else if let Some(dialog) = item.as_str() {
                        dialogs.push(vec![dialog.to_string()]);
                    }
                }
                Some(dialogs)
            } else {
                None
            };

        let quests = if json["quests"].is_array() && !json["quests"].is_empty() {
            let mut quests = Vec::new();
            for quest in json["quests"].members() {
                if let Some(quest_id) = quest.as_str() {
                    if quests.iter().any(|q: &String| q == quest_id) {
                        // checks if the quest is already added
                        return None;
                    }
                    else {
                        // normal case
                        quests.push(quest_id.to_string());
                    }
                }
            }
            Some(quests)
        } else {
            None
        };

        let mut npc_type = 0;
        if dialogs.is_some() {
            npc_type |= NPC_TALKER;
        }
        if quests.is_some() {
            npc_type |= NPC_QUEST_GIVER;
        }
        if max_hp.is_some() {
            npc_type |= NPC_MOB;
        }

        if npc_type == 0 {
            error!("invalid npc type: no dialogs, quests or max_hp");
            return None;
        }

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

    pub fn set_hp(&mut self, hp: Option<u32>) {
        self.hp = hp;
    }
    pub fn get_dialog(&self, index: usize) -> Option<&Dialog> {
        self.dialogs.as_ref()?.get(index)
    }
    pub fn get_spawn_room(&self) -> &str {
        &self.room_spawn
    }
    pub fn parse_protocol_representation(protocol_name: &str) -> Option<(NpcId, String)> {
        if let Some((id, name)) = protocol_name.split_once('.') {
            id.parse::<NpcId>().ok().map(|id| (id, name.to_string()))
        } else {
            None
        }
    }
    pub fn get_protocol_representation(&self) -> String {
        format!("{}.{}", self.id, self.name)
    }
    pub fn get_dialogs(&self) -> Option<&Vec<Dialog>> {
        self.dialogs.as_ref()
    }
    pub fn get_quests(&self) -> Option<&Vec<Questid>> {
        self.quests.as_ref()
    }
}
