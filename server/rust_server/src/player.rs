use crate::groups::GroupId;
use crate::inventory::Inventory;
use std::collections::HashSet;
use crate::items::ItemId;

pub type PlayerId = u32;
pub type PlayerCount = u32;

pub struct Player {
    name: String, 
    id: PlayerId,
    group_id: Option<GroupId>,
    inventory: Inventory
}

impl Player {
    pub fn new(name: String, id: PlayerId) -> Self {
        Self { name, id, group_id: None, inventory: Inventory::new() }
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
}