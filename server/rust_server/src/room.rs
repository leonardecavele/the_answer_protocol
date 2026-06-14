use crate::inventory::Inventory;
use crate::items::ItemId;
pub struct Room {
    name: String,
    all_items: Inventory,
}

impl Room {
    pub fn new(name: String) -> Self {
        Self {
            name,
            all_items: Inventory::new(),
        }
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
