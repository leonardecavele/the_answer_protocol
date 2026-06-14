use crate::groups::GroupId;
use crate::inventory::Inventory;
use crate::constantes::HARDCODED_PLAYER_ROOM;
use crate::items::ItemId;
use std::collections::HashSet;

pub type PlayerId = u32;
pub type PlayerCount = u32;

pub struct Player {
    name: String,
    id: PlayerId,
    group_id: Option<GroupId>,
    inventory: Inventory,
    current_room: String,
}

impl Player {
    pub fn new(name: String, id: PlayerId) -> Self {
        Self {
            name,
            id,
            group_id: None,
            inventory: Inventory::new(),
            current_room: HARDCODED_PLAYER_ROOM.to_string(),
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
    pub fn get_group_id(&self) -> Option<GroupId> {
        self.group_id
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
}
