use super::Sprite;
use crate::data::manifest::Manifest;

pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sprite: Sprite,
}

impl Item {
    pub fn from_manifest(id: String, manifest: &Manifest) -> Self {
        match manifest.items.get(&id) {
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
