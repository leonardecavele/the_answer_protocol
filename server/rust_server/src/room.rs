use crate::constantes::ExitDir;
use crate::inventory::Inventory;
use crate::items::ItemId;
use std::collections::HashMap;
pub type RoomName = String;

pub struct Room {
    name: String,
    all_items: Inventory,
    exits: HashMap<ExitDir, RoomName>,
}

impl Room {
    pub fn new(name: String, exits: HashMap<ExitDir, RoomName>) -> Self {
        Self {
            name,
            all_items: Inventory::new(),
            exits,
        }
    }
    pub fn has_exit(&self, dir: ExitDir) -> bool {
        self.exits.contains_key(&dir)
    }
    pub fn add_item(&mut self, item_id: ItemId) {
        self.all_items.add_item(item_id);
    }
    pub fn remove_item(&mut self, item_id: ItemId) {
        self.all_items.remove_item(item_id);
    }
    pub fn contains_item(&self, item_id: ItemId) -> bool {
        self.all_items.contains_item(item_id)
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
