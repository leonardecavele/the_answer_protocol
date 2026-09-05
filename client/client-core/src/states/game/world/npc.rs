use super::Sprite;
use crate::manifest::{Manifest, NpcKind};

pub struct Npc {
    pub id: String,
    pub name: String,
    pub kind: NpcKind,
    pub sprite: Sprite,
}

impl Npc {
    pub fn from_manifest(id: String, manifest: &Manifest) -> Self {
        match manifest.npcs.get(&id) {
            Some(entry) => Self {
                name: entry.name.clone(),
                kind: entry.kind.clone(),
                sprite: Sprite::from(entry),
                id,
            },
            None => Self {
                name: id.clone(),
                kind: NpcKind::Normal,
                sprite: Sprite::None,
                id,
            },
        }
    }
}
