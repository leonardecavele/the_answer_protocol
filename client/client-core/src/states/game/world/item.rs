use super::Sprite;
use crate::manifest::Manifest;

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
