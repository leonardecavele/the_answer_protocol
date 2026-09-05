use crate::items::ItemId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Inventory {
    #[serde(default)]
    items: Vec<ItemId>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    pub fn contains_item(&self, item_id: ItemId) -> bool {
        self.items.contains(&item_id)
    }

    pub fn add_item(&mut self, item_id: ItemId) {
        self.items.push(item_id);
    }

    pub fn remove_item(&mut self, item_id: ItemId) {
        if let Some(pos) = self.items.iter().position(|&x| x == item_id) {
            self.items.remove(pos);
        }
    }

    pub fn get_items(&self) -> &Vec<ItemId> {
        &self.items
    }
    pub fn get_items_mut(&mut self) -> &mut Vec<ItemId> {
        &mut self.items
    }
}
