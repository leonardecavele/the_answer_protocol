use std::time::Instant;

use crate::room::RoomId;

pub type ItemId = u64;

#[derive(Clone, Debug)]
pub struct SpawnInfo {
    pub room: String,
    pub cooldown: u64,
    pub timer: Instant,
}

impl SpawnInfo {
    pub fn new(room: String, cooldown: u64) -> Self {
        Self {
            room,
            cooldown,
            timer: Instant::now(),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.timer.elapsed().as_secs() >= self.cooldown
    }

    pub fn reset_timer(&mut self) {
        self.timer = Instant::now();
    }
}

#[derive(Clone)]
pub struct Item {
    id: ItemId,
    model_id: ItemId,
    name: String,
    description: String,
    dropped_at: Option<Instant>,
    remove_despawn_in_room: Option<RoomId>,
    spawn_info: Option<SpawnInfo>,
}

impl Item {
    pub fn new(id: ItemId, name: String, description: String, spawn_info: Option<SpawnInfo>) -> Self {
        Self {
            id,
            model_id: id,
            name,
            description,
            dropped_at: None,
            remove_despawn_in_room: None,
            spawn_info,
        }
    }

    pub fn get_model_id(&self) -> ItemId {
        self.model_id
    }

    pub fn clone_as_instance(&self, new_id: ItemId) -> Self {
        let mut new_item = self.clone();
        new_item.id = new_id;
        new_item.spawn_info = None;
        // The model_id remains the same as the original base item
        new_item
    }
    pub fn get_remove_despawn_in_room(&self) -> Option<RoomId> {
        self.remove_despawn_in_room
    }
    pub fn remove_despawn_in_room(&mut self, room_id: RoomId) {
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
    
    pub fn get_spawn_info(&self) -> Option<&SpawnInfo> {
        self.spawn_info.as_ref()
    }

    pub fn get_spawn_info_mut(&mut self) -> Option<&mut SpawnInfo> {
        self.spawn_info.as_mut()
    }

    pub fn can_spawn(&self) -> bool {
        self.spawn_info.as_ref().map_or(false, |s| s.is_ready())
    }

    pub fn reset_spawn_timer(&mut self) {
        if let Some(s) = self.spawn_info.as_mut() {
            s.reset_timer();
        }
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
