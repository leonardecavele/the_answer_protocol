use std::time::Instant;

use crate::room::RoomId;

pub type ItemId = u64;

#[derive(Clone)]
pub struct Item {
    id: ItemId,
    name: String,
    description: String,
    dropped_at: Option<Instant>,
    remove_despawn_in_room: Option<RoomId>,
}

impl Item {
    pub fn new(id: ItemId, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            dropped_at: None,
            remove_despawn_in_room: None,
        }
    }
    pub fn get_remove_despawn_in_room(&self) -> Option<RoomId> {
        self.remove_despawn_in_room
    }
    pub fn set_remove_despawn_in_room(&mut self, room_id: RoomId) {
        self.remove_despawn_in_room = Some(room_id);
    }
    pub fn get_dropped_at(&self) -> Option<Instant> {
        self.dropped_at
    }
    pub fn set_dropped_at(&mut self, dropped_at: Instant) {
        self.dropped_at = Some(dropped_at);
    }
    pub fn stop_dropped_at(&mut self) {
        self.dropped_at = None;
    }
    pub fn get_id(&self) -> ItemId {
        self.id
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn parse_item(item: &str) -> Option<(ItemId, String)> {
        let parts: Vec<_> = item.split('.').collect();
        if !(parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()) {
            return None;
        }
        Some((parts[0].parse::<ItemId>().ok()?, parts[1].to_string()))
    }

    pub fn protocol_representation(item_id: ItemId, item_name: &str) -> String {
        format!("{}.{}", item_id, item_name)
    }

    pub fn get_protocol_representation(&self) -> String {
        Self::protocol_representation(self.id, &self.name)
    }
}
