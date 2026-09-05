use crate::data::manifest::{ItemEntry, Manifest, NpcEntry, RoomEntry};
use std::time::Duration;

pub enum Sprite {
    None,
    Static(String),
    Animated {
        image_paths: Vec<String>,
        frame_ms: u64,
    },
}

impl Sprite {
    pub fn of_npc(id: &str, manifest: &Manifest) -> Sprite {
        manifest
            .npcs
            .get(id)
            .map(Sprite::from)
            .unwrap_or(Sprite::None)
    }

    pub fn of_room(id: &str, manifest: &Manifest) -> Sprite {
        manifest
            .rooms
            .get(id)
            .map(Sprite::from)
            .unwrap_or(Sprite::None)
    }

    pub fn frame_at(&self, elapsed: Duration) -> Option<&str> {
        match self {
            Sprite::None => None,
            Sprite::Static(image_path) => Some(image_path),
            Sprite::Animated {
                image_paths,
                frame_ms,
            } if !image_paths.is_empty() && *frame_ms > 0 => {
                let index = (elapsed.as_millis() as u64 / frame_ms) as usize % image_paths.len();
                Some(&image_paths[index])
            }
            Sprite::Animated { .. } => None,
        }
    }
}

impl From<&NpcEntry> for Sprite {
    fn from(npc_entry: &NpcEntry) -> Self {
        if let (Some(image_paths), Some(frame_ms)) =
            (npc_entry.image_paths.clone(), npc_entry.frame_ms)
        {
            Self::Animated {
                image_paths,
                frame_ms,
            }
        } else if let Some(image_path) = npc_entry.image_path.clone() {
            Self::Static(image_path)
        } else {
            Self::None
        }
    }
}

impl From<&ItemEntry> for Sprite {
    fn from(item_entry: &ItemEntry) -> Self {
        match item_entry.image_path.clone() {
            Some(image_path) => Self::Static(image_path),
            None => Self::None,
        }
    }
}

impl From<&RoomEntry> for Sprite {
    fn from(room_entry: &RoomEntry) -> Self {
        match room_entry.image_path.clone() {
            Some(image_path) => Self::Static(image_path),
            None => Self::None,
        }
    }
}
