use crate::collections::SelectableList;
use crate::states::game::world::{Item, Npc};
use std::collections::HashMap;

pub struct RoomState {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub exits: HashMap<String, String>,
    pub players: Vec<String>,
    pub npcs: SelectableList<Npc>,
    pub items: SelectableList<Item>,
}

impl RoomState {
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            description: None,
            exits: HashMap::new(),
            players: Vec::new(),
            npcs: SelectableList::new(),
            items: SelectableList::new(),
        }
    }

    pub fn clear(&mut self) {
        self.id = None;
        self.name = None;
        self.description = None;
        self.exits.clear();
        self.players.clear();
        self.npcs.clear();
        self.items.clear();
    }

    pub fn has_exit(&self, direction: &str) -> bool {
        self.exits.contains_key(direction)
    }

    pub fn has_item(&self, id: &str) -> bool {
        self.items.iter().any(|item| item.id == id)
    }

    pub fn take_item(&mut self, id: &str) -> Option<Item> {
        let index = self.items.iter().position(|item| item.id == id)?;
        self.items.remove(index)
    }
}

impl Default for RoomState {
    fn default() -> Self {
        Self::new()
    }
}
