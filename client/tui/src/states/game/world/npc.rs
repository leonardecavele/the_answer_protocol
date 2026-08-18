use crate::data::manifest::{Manifest, NpcKind};
use super::Sprite;

pub struct Npc {
    pub id: String,
    pub name: String,
    pub kind: NpcKind,
    pub actions: Vec<String>,
    pub sprite: Sprite,
}

impl Npc {
    pub fn from_manifest(id: String, manifest: &Manifest) -> Self {
        match manifest.npcs.get(&id) {
            Some(entry) => Self {
                name: entry.name.clone(),
                kind: entry.kind.clone(),
                actions: entry.actions.clone(),
                sprite: Sprite::from(entry),
                id,
            },
            None => Self {
                name: id.clone(),
                kind: NpcKind::Normal,
                actions: Vec::new(),
                sprite: Sprite::None,
                id,
            },
        }
    }
}
