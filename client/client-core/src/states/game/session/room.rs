use crate::collections::SelectableList;
use crate::states::game::Direction;
use crate::states::game::world::{DIRECTION_COUNT, Item, Npc};
use std::collections::HashMap;
use tracing::warn;

#[derive(Debug, Default)]
pub struct Exits([Option<String>; DIRECTION_COUNT]);

impl Exits {
    pub fn get(&self, direction: Direction) -> Option<&str> {
        self.0[direction.quarter_turns()].as_deref()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Direction, &str)> {
        Direction::CLOCKWISE
            .into_iter()
            .filter_map(|direction| self.get(direction).map(|to| (direction, to)))
    }
}

impl From<HashMap<String, String>> for Exits {
    fn from(value: HashMap<String, String>) -> Self {
        let mut exits: Exits = Exits::default();

        for (key, to) in value.into_iter() {
            if let Some(direction) = Direction::from_key(key.as_str()) {
                exits.0[direction.quarter_turns()] = Some(to);
            } else {
                warn!("unknown exit direction: {}", key)
            }
        }

        exits
    }
}

pub struct Room {
    pub id: String,
    pub name: String,
    pub description: String,
    pub exits: Exits,
    pub players: SelectableList<String>,
    pub npcs: SelectableList<Npc>,
    pub items: SelectableList<Item>,
}

impl Room {
    pub fn has_exit(&self, direction: Direction) -> bool {
        self.exits.get(direction).is_some()
    }

    pub fn has_item(&self, id: &str) -> bool {
        self.items.iter().any(|item| item.id == id)
    }

    pub fn take_item(&mut self, id: &str) -> Option<Item> {
        let index = self.items.iter().position(|item| item.id == id)?;
        self.items.remove(index)
    }

    pub fn spawn_item(&mut self, item: Item) {
        self.items.push(item);
    }

    pub fn spawn_npc(&mut self, npc: Npc) {
        self.npcs.push(npc);
    }

    pub fn remove_npc(&mut self, id: &str) {
        self.npcs.retain(|npc| npc.id != id);
    }

    pub fn player_entered(&mut self, name: String) {
        if !self.players.contains(&name) {
            self.players.push(name);
        }
    }

    pub fn player_left(&mut self, name: &str) {
        self.players.retain(|player| player != name);
    }
}
