use crate::constantes::{
    PLAYER_ROOM_SPAWN, NO_MORE_MESSAGES, PLAYER_STARTING_HP, PLAYER_STARTING_MAX_HP,
};
use crate::inventory::Inventory;
use crate::items::ItemId;
use crate::npc::Npc;
use crate::room::RoomName;
use std::collections::{HashMap, HashSet};
use rand::RngExt;

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
}

impl Player {
    pub fn new(name: String, id: PlayerId) -> Self {
        Self {
            name,
            id,
            hp: PLAYER_STARTING_HP,
            max_hp: PLAYER_STARTING_MAX_HP,
            inventory: Inventory::new(),
            current_room: PLAYER_ROOM_SPAWN.to_string(),
            dialogs_index: HashMap::new(),
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

    pub fn get_items(&self) -> &HashSet<ItemId> {
        &self.inventory.get_items()
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
    pub fn move_to_room(&mut self, room: &RoomName) {
        self.current_room = room.clone();
    }
    pub fn get_dialog_index_for_npc(&self, npc_name: &str) -> Option<(usize, usize)> {
        self.dialogs_index.get(npc_name).copied()
    }
    pub fn talk_with(&mut self, npc: &Npc) -> String {
        let dialogs = match npc.get_dialogs() {
            Some(d) if !d.is_empty() => d,
            _ => return NO_MORE_MESSAGES.to_string(),
        };

        let (dialog_list_index, current_line_index) = self.dialogs_index
            .get(&npc.get_name())
            .copied()
            .unwrap_or_else(|| {
                let mut rng = rand::rng();
                let random_index = rng.random_range(0..dialogs.len());
                (random_index, 0)
            // change dialog vec if we are at end of dialogue
            });

        if dialog_list_index >= dialogs.len() {
            return NO_MORE_MESSAGES.to_string();
        }

        let current_dialog_list = &dialogs[dialog_list_index];

        if current_line_index >= current_dialog_list.len() {
            self.dialogs_index.remove(&npc.get_name());
            return NO_MORE_MESSAGES.to_string();
        }

        let message = &current_dialog_list[current_line_index];
        self.dialogs_index.insert(npc.get_name(), (dialog_list_index, current_line_index + 1));

        message.to_string()
    }
    pub fn has_item(&self, item_id: ItemId) -> bool {
        self.inventory.contains_item(item_id)
    }
}
