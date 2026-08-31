use crate::items::ItemId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Serialize, Deserialize)]
pub struct Inventory {
    items: HashSet<ItemId>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: HashSet::new(),
        }
    }

    pub fn contains_item(&self, item_id: ItemId) -> bool {
        self.items.contains(&item_id)
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
