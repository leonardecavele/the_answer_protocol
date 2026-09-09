use crate::constants::{
    NO_MORE_MESSAGES, PLAYER_ROOM_SPAWN, PLAYER_STARTING_HP, PLAYER_STARTING_MAX_HP,
};
use crate::inventory::Inventory;
use crate::items::ItemId;
use crate::npc::Npc;
use crate::quests::{Loot, Questid};
use crate::room::RoomName;
use crate::save::Save;
use rand::RngExt;
use std::collections::HashMap;

pub type PlayerId = u32;
pub type PlayerCount = u32;

pub struct Player {
    name: String,
    id: PlayerId,
    hp: u32,
    max_hp: u32,
    inventory: Inventory,
    current_room: String,
    dialogs_index: HashMap<String, (usize, usize)>,
    completed_quests: HashMap<Questid, Vec<Vec<Loot>>>,
    pub last_rooms: Vec<String>,
}

impl Player {
    pub fn new(name: String, id: PlayerId) -> Self {
        Self {
            name,
            id,
            hp: PLAYER_STARTING_HP,
            max_hp: PLAYER_STARTING_MAX_HP,
            inventory: Inventory::new(),
            current_room: PLAYER_ROOM_SPAWN.to_owned(),
            dialogs_index: HashMap::new(),
            completed_quests: HashMap::new(),
            last_rooms: vec![PLAYER_ROOM_SPAWN.to_owned()],
        }
    }
    pub fn reset(&mut self) {
        self.hp = PLAYER_STARTING_HP;
        self.max_hp = PLAYER_STARTING_MAX_HP;
        self.inventory = Inventory::new();
        self.current_room = PLAYER_ROOM_SPAWN.to_owned();
        self.dialogs_index.clear();
        self.completed_quests.clear();
        self.last_rooms = vec![PLAYER_ROOM_SPAWN.to_owned()];
    }

    pub fn from_save(save: Save) -> Self {
        let current_room = save.current_room;
        Self {
            name: save.name,
            id: save.id,
            hp: save.hp,
            max_hp: save.max_hp,
            inventory: save.inventory,
            current_room: current_room.clone(),
            dialogs_index: HashMap::new(),
            completed_quests: save.completed_quests,
            last_rooms: vec![current_room],
        }
    }
    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_id(&self) -> PlayerId {
        self.id
    }
    pub fn get_hp(&self) -> u32 {
        self.hp
    }
    pub fn get_max_hp(&self) -> u32 {
        self.max_hp
    }
    pub fn set_hp(&mut self, hp: u32) {
        self.hp = hp;
    }

    pub fn get_items(&self) -> &Vec<ItemId> {
        self.inventory.get_items()
    }
    pub fn add_item(&mut self, item_id: ItemId) {
        self.inventory.add_item(item_id);
    }
    pub fn remove_item(&mut self, item_id: ItemId) {
        self.inventory.remove_item(item_id);
    }
    pub fn get_current_room(&self) -> &str {
        &self.current_room
    }
    pub fn get_inventory(&self) -> &Inventory {
        &self.inventory
    }
    pub fn move_to_room(&mut self, room: &RoomName) {
        self.current_room = room.clone();
        self.add_last_room(room.clone());
    }
    pub fn get_last_rooms(&self) -> &[String] {
        &self.last_rooms
    }
    pub fn get_last_rooms_mut(&mut self) -> &mut Vec<String> {
        &mut self.last_rooms
    }
    pub fn add_last_room(&mut self, room: String) {
        if self.last_rooms.last() == Some(&room) {
            return;
        }
        self.last_rooms.push(room);
        if self.last_rooms.len() > 7 {
            self.last_rooms.remove(0);
        }
    }
    pub fn clear_last_rooms(&mut self) {
        self.last_rooms.clear();
    }
    pub fn reset_last_rooms_to_current(&mut self) {
        let current = self.current_room.clone();
        self.last_rooms.clear();
        self.last_rooms.push(current);
    }
    pub fn get_dialog_index_for_npc(&self, npc_name: &str) -> Option<(usize, usize)> {
        self.dialogs_index.get(npc_name).copied()
    }
    pub fn talk_with(&mut self, npc: &Npc) -> String {
        let dialogs = match npc.get_dialogs() {
            Some(d) if !d.is_empty() => d,
            _ => return NO_MORE_MESSAGES.to_owned(),
        };

        let (dialog_list_index, current_line_index) = self
            .dialogs_index
            .get(&npc.get_name())
            .copied()
            .unwrap_or_else(|| {
                let mut rng = rand::rng();
                let random_index = rng.random_range(0..dialogs.len());
                (random_index, 0)
                // change dialog vec if we are at end of dialogue
            });

        if dialog_list_index >= dialogs.len() {
            return NO_MORE_MESSAGES.to_owned();
        }

        let current_dialog_list = &dialogs[dialog_list_index];

        if current_line_index >= current_dialog_list.len() {
            self.dialogs_index.remove(&npc.get_name());
            return NO_MORE_MESSAGES.to_owned();
        }

        let message = &current_dialog_list[current_line_index];
        self.dialogs_index
            .insert(npc.get_name(), (dialog_list_index, current_line_index + 1));

        message.to_owned()
    }
    pub fn has_item(&self, item_id: ItemId) -> bool {
        self.inventory.contains_item(item_id)
    }
    pub fn get_completed_quests(&self) -> &HashMap<Questid, Vec<Vec<Loot>>> {
        &self.completed_quests
    }
    pub fn get_completed_quests_mut(&mut self) -> &mut HashMap<Questid, Vec<Vec<Loot>>> {
        &mut self.completed_quests
    }
    pub fn add_completed_quest(&mut self, quest_name: Questid, loots: Vec<Loot>) {
        self.completed_quests
            .entry(quest_name)
            .or_default()
            .push(loots);
    }
    pub fn set_completed_quests(&mut self, completed_quests: HashMap<Questid, Vec<Vec<Loot>>>) {
        self.completed_quests = completed_quests;
    }
}
