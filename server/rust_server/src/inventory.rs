use crate::items::ItemId;
use std::collections::HashSet;
// use json::object;

#[derive(Clone)]
pub struct Inventory {
    items: HashSet<ItemId>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: HashSet::new(),
        }
    }

    pub fn contains_item(&self, item_id: ItemId) -> bool {
        return self.items.contains(&item_id);
    }

    pub fn add_item(&mut self, item_id: ItemId) {
        self.items.insert(item_id);
    }

    pub fn remove_item(&mut self, item_id: ItemId) {
        self.items.remove(&item_id);
    }

    pub fn get_items(&self) -> &HashSet<ItemId> {
        &self.items
    }
    pub fn get_items_mut(&mut self) -> &mut HashSet<ItemId> {
        &mut self.items
    }
}
