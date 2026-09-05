use crate::collections::SelectableList;
use crate::states::game::world::Item;
use api_client::commands::QuestData;

pub struct PlayerState {
    pub name: Option<String>,
    pub hp: u32,
    pub max_hp: u32,
    pub inventory: SelectableList<Item>,
    pub quests: SelectableList<QuestData>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            name: None,
            hp: 100,
            max_hp: 100,
            inventory: SelectableList::new(),
            quests: SelectableList::new(),
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name.to_uppercase());
    }

    pub fn is_me(&self, player_name: &str) -> bool {
        self.name
            .as_deref()
            .is_some_and(|name| name.eq_ignore_ascii_case(player_name))
    }

    pub fn set_vitals(&mut self, hp: u32, max_hp: u32) {
        self.max_hp = max_hp;
        self.hp = hp.min(max_hp);
    }

    pub fn set_hp(&mut self, hp: u32) {
        self.hp = hp.min(self.max_hp);
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.hp = self.hp.saturating_sub(amount);
    }

    pub fn is_dead(&self) -> bool {
        self.hp == 0
    }

    pub fn has_item(&self, id: &str) -> bool {
        self.inventory.iter().any(|item| item.id == id)
    }

    pub fn take_item(&mut self, id: &str) -> Option<Item> {
        let index = self.inventory.iter().position(|item| item.id == id)?;
        self.inventory.remove(index)
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}
