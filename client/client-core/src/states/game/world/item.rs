use super::Sprite;
use crate::manifest::Manifest;
use std::ops::Deref;

pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sprite: Sprite,
}

impl Item {
    pub fn from_manifest(id: String, manifest: &Manifest) -> Self {
        let (_, item_name) = id.split_once('.').unwrap_or(("", id.as_str()));

        match manifest.items.get(item_name) {
            Some(entry) => Self {
                name: entry.name.clone(),
                description: entry.description.clone(),
                sprite: Sprite::from(entry),
                id,
            },
            None => Self {
                name: id.clone(),
                description: "No description available.".to_string(),
                sprite: Sprite::None,
                id,
            },
        }
    }
}

pub struct ItemStack {
    pub name: String,
    items: Vec<Item>,
}

impl ItemStack {
    pub fn new(item: Item) -> Self {
        Self {
            name: item.name.clone(),
            items: vec![item],
        }
    }

    pub fn push(&mut self, item: Item) {
        self.items.push(item);
    }

    pub fn take_item(&mut self, id: &str) -> Option<Item> {
        let index = self.items.iter().position(|item| item.id == id)?;

        Some(self.items.remove(index))
    }
}

impl Deref for ItemStack {
    type Target = [Item];

    fn deref(&self) -> &[Item] {
        &self.items
    }
}
