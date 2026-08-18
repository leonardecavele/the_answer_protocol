use crate::collections::SelectableList;
use api_client::commands::QuestData;

pub struct PlayerState {
    pub name: Option<String>,
    pub hp: u32,
    pub max_hp: u32,
    pub inventory: SelectableList<String>,
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

    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.hp = self.hp.saturating_sub(amount);
    }

    pub fn is_dead(&self) -> bool {
        self.hp == 0
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}
