use tracing::error;

use crate::constants::{LOST_ITEM, LOST_ITEM_SPAWN_ID, PLAYER_ROOM_SPAWN};
use crate::items::{Item, ItemId};
use crate::npc::{Npc, NpcId};
use crate::quests::{Quest, Questid};
use crate::room::{Room, RoomId};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;

pub struct Parser {
    npcs: HashMap<NpcId, Npc>,
    ncps_file: String,
    items: HashMap<ItemId, Item>,
    items_file: String,
    rooms: HashMap<RoomId, Room>,
    rooms_file: String,
    quests: HashMap<Questid, Quest>,
    quests_file: String,
}

impl Parser {
    pub fn new(npcs_path: &str, items_path: &str, rooms_path: &str, quests_path: &str) -> Self {
        Self {
            npcs: HashMap::new(),
            ncps_file: npcs_path.to_string(),
            items: HashMap::new(),
            items_file: items_path.to_string(),
            rooms: HashMap::new(),
            rooms_file: rooms_path.to_string(),
            quests: HashMap::new(),
            quests_file: quests_path.to_string(),
        }
    }

    pub fn parse_all(&mut self) {
        if let Err(e) = self.parse_quests() {
            error!("{}", e);
            std::process::exit(1);
        }
        if let Err(e) = self.parse_npcs() {
            error!("{}", e);
            std::process::exit(1);
        }
        if let Err(e) = self.parse_items() {
            error!("{}", e);
            std::process::exit(1);
        }
        if let Err(e) = self.parse_rooms() {
            error!("{}", e);
            std::process::exit(1);
        }
        if let Err(e) = self.check_good_ids() {
            error!("{}", e);
            std::process::exit(1);
        }
    }

    pub fn check_good_ids(&self) -> Result<(), String> {
        let mut valid_room_names = HashSet::new();
        for room in self.rooms.values() {
            valid_room_names.insert(room.get_name().to_string());
        }

        if !valid_room_names.contains(PLAYER_ROOM_SPAWN) {
            return Err(format!(
                "PLAYER_ROOM_SPAWN '{}' is not defined",
                PLAYER_ROOM_SPAWN
            ));
        }

        for (room_id, room) in &self.rooms {
            for item_id in room.get_inventory().get_items() {
                if !self.items.contains_key(item_id) {
                    return Err(format!(
                        "Room {} ({}) contains invalid item_id {}",
                        room_id,
                        room.get_name(),
                        item_id
                    ));
                }
            }
            for dest in room.get_exits().values() {
                if !valid_room_names.contains(dest) {
                    return Err(format!(
                        "Room {} ({}) has an exit to invalid room name '{}'",
                        room_id,
                        room.get_name(),
                        dest
                    ));
                }
            }
        }

        for (npc_id, npc) in &self.npcs {
            if !valid_room_names.contains(npc.get_spawn_room()) {
                return Err(format!(
                    "NPC {} ({}) spawns in invalid room '{}'",
                    npc_id,
                    npc.get_name(),
                    npc.get_spawn_room()
                ));
            }
            if let Some(quests) = npc.get_quests() {
                for quest_id in quests {
                    if !self.quests.contains_key(quest_id) {
                        return Err(format!(
                            "NPC {} ({}) has invalid quest_id '{}'",
                            npc_id,
                            npc.get_name(),
                            quest_id
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn parse_quests(&mut self) -> Result<(), String> {
        let content = fs::read_to_string(&self.quests_file)
            .map_err(|e| format!("Failed to read file '{}': {}", self.quests_file, e))?;

        let parsed = json::parse(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut quests = HashMap::new();

        if parsed["quests"].is_array() {
            for item in parsed["quests"].members() {
                if let Some(quest) = Quest::new(item) {
                    if quests.contains_key(quest.get_name()) {
                        return Err(format!("duplicate quest: {}", quest.get_name()));
                    }
                    quests.insert(quest.get_name().to_string(), quest);
                } else {
                    return Err(format!("invalid quest: {}", item));
                }
            }
        } else {
            return Err("JSON does not contain 'quests' array".to_string());
        }

        self.quests = quests;
        Ok(())
    }

    pub fn get_quests(&self) -> &HashMap<Questid, Quest> {
        &self.quests
    }

    pub fn parse_npcs(&mut self) -> Result<(), String> {
        let content = fs::read_to_string(&self.ncps_file)
            .map_err(|e| format!("Failed to read file '{}': {}", self.ncps_file, e))?;

        let parsed = json::parse(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut npcs = HashMap::new();
        let mut npc_id: NpcId = 0;
        if parsed["npcs"].is_array() {
            for member in parsed["npcs"].members() {
                if let Some(npc) = Npc::new(member, npc_id) {
                    npcs.insert(npc_id, npc);
                } else {
                    return Err(format!("invalid npc: {}", member));
                }
                npc_id += 1;
            }
        } else {
            return Err("JSON does not contain 'npcs' array".to_string());
        }

        self.npcs = npcs;
        Ok(())
    }
    pub fn get_npcs(&self) -> &HashMap<NpcId, Npc> {
        &self.npcs
    }

    pub fn parse_items(&mut self) -> Result<(), String> {
        let content = fs::read_to_string(&self.items_file)
            .map_err(|e| format!("Failed to read file '{}': {}", self.items_file, e))?;

        let parsed = json::parse(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut items = HashMap::new();
        let mut id: u64 = 0;
        if parsed["items"].is_array() {
            for item in parsed["items"].members() {
                let name = item["name"].as_str().ok_or("item name must be a string")?;
                let description = item["description"]
                    .as_str()
                    .ok_or("item description must be a string")?;

                let mut parsed_item = Item::new(id, name.to_string(), description.to_string());
                if parsed_item.get_id() == LOST_ITEM as ItemId {
                    let room_id = LOST_ITEM_SPAWN_ID;
                    parsed_item.remove_despawn_in_room(room_id);
                }

                items.insert(parsed_item.get_id(), parsed_item);
                id += 1;
            }
        } else {
            return Err("JSON does not contain 'items' array".to_string());
        }

        self.items = items;
        Ok(())
    }

    pub fn get_items(&self) -> &HashMap<ItemId, Item> {
        &self.items
    }

    pub fn parse_rooms(&mut self) -> Result<(), String> {
        let content = fs::read_to_string(&self.rooms_file)
            .map_err(|e| format!("Failed to read file '{}': {}", self.rooms_file, e))?;

        let parsed = json::parse(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut rooms = HashMap::new();
        let mut room_id: RoomId = 0;
        if parsed["rooms"].is_array() {
            for room in parsed["rooms"].members() {
                let name = room["name"].as_str().ok_or("room name must be a string")?;
                let description = room["description"]
                    .as_str()
                    .ok_or("room description must be a string")?;

                let mut exits = HashMap::new();
                if room["exits"].is_object() {
                    for (dir, dest) in room["exits"].entries() {
                        let dir_lower = dir.to_lowercase();
                        if !["north", "east", "south", "west"].contains(&dir_lower.as_str()) {
                            return Err(format!(
                                "invalid exit direction '{}' in room {}",
                                dir, room_id
                            ));
                        }
                        if let Some(dest_str) = dest.as_str() {
                            exits.insert(dir.to_uppercase(), dest_str.to_string());
                        } else {
                            return Err(format!(
                                "destination for exit '{}' must be a string in room {}",
                                dir, room_id
                            ));
                        }
                    }
                } else if !room["exits"].is_null() && !room["exits"].is_empty() {
                    return Err(format!("exits must be an object in room {}", room_id));
                }

                let mut parsed_room =
                    Room::new(room_id, name.to_string(), description.to_string(), exits);

                if room["items"].is_array() {
                    for item_id_json in room["items"].members() {
                        let item_id_val =
                            item_id_json.as_i64().ok_or("room item must be an int")?;
                        if item_id_val < 0 {
                            return Err(format!(
                                "room item id must be a positive int in room {}",
                                room_id
                            ));
                        }
                        parsed_room.add_item(item_id_val as ItemId);
                    }
                } else if !room["items"].is_null() && !room["items"].is_empty() {
                    return Err(format!("items must be an array in room {}", room_id));
                }

                rooms.insert(parsed_room.get_id(), parsed_room);
                room_id += 1;
            }
        } else {
            return Err("JSON does not contain 'rooms' array".to_string());
        }

        self.rooms = rooms;
        Ok(())
    }

    pub fn get_rooms(&self) -> &HashMap<RoomId, Room> {
        &self.rooms
    }
}
