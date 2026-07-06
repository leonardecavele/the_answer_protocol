use crate::constantes::Direction;
use crate::inventory::Inventory;
use crate::items::ItemId;
use std::collections::HashMap;
pub type RoomName = String;
pub type RoomId = u32;
#[derive(Clone)]
pub struct Room {
    id: RoomId,
    name: String,
    description: String,
    protocol_representation: String,
    all_items: Inventory,
    exits: HashMap<Direction, RoomName>,
}

impl Room {
    pub fn new(
        id: RoomId,
        name: String,
        description: String,
        exits: HashMap<Direction, RoomName>,
    ) -> Self {
        let protocol_representation = Room::protocol_representation(id, name.clone());
        Self {
            id,
            name,
            description,
            protocol_representation,
            all_items: Inventory::new(),
            exits,
        }
    }
    pub fn get_id_from_protocol_representation(protocol_representation: &str) -> RoomId {
        return protocol_representation
            .split('.')
            .next()
            .unwrap()
            .parse::<RoomId>()
            .unwrap();
    }
    pub fn get_protocol_representation(&self) -> &str {
        &self.protocol_representation
    }
    pub fn get_exits(&self) -> &HashMap<Direction, RoomName> {
        &self.exits
    }
    pub fn get_id(&self) -> RoomId {
        self.id
    }
    pub fn has_exit(&self, dir: Direction) -> bool {
        self.exits.contains_key(&dir)
    }
    pub fn add_item(&mut self, item_id: ItemId) {
        self.all_items.add_item(item_id);
    }
    pub fn remove_item(&mut self, item_id: ItemId) {
        self.all_items.remove_item(item_id);
    }
    pub fn contains_item(&self, item_id: ItemId) -> bool {
        self.all_items.contains_item(item_id)
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_inventory(&self) -> &Inventory {
        &self.all_items
    }
    pub fn get_description(&self) -> &str {
        &self.description
    }
    pub fn get_neighbor_room_name(&self, dir: &Direction) -> Option<&RoomName> {
        self.exits.get(dir)
    }
    pub fn protocol_representation(id: RoomId, name: RoomName) -> String {
        return format!("{}.{}", id, name);
    }
    pub fn get_all_items(&mut self) -> &mut Inventory {
        &mut self.all_items
    }
    pub fn set_inventory(&mut self, inventory: Inventory) {
        self.all_items = inventory;
    }
}
