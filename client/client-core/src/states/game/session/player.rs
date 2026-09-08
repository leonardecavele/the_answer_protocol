use crate::collections::SelectableList;
use crate::states::game::session::quest::{Quest, QuestId};
use crate::states::game::world::Item;
use client_api::commands::{QuestData, QuestStatus};

pub struct PlayerState {
    pub name: Option<String>,
    pub hp: u32,
    pub max_hp: u32,
    pub inventory: SelectableList<Item>,
    pub quests: SelectableList<Quest>,
    next_quest_id: u64,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            name: None,
            hp: 100,
            max_hp: 100,
            inventory: SelectableList::new(),
            quests: SelectableList::new(),
            next_quest_id: 0,
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

    fn new_quest(&mut self, data: QuestData) -> Quest {
        self.next_quest_id += 1;

        Quest::new(self.next_quest_id, data)
    }

    pub fn find_quest(&self, id: QuestId) -> Option<&Quest> {
        self.quests.iter().find(|quest| quest.id == id)
    }

    fn find_active_quest_index(&self, name: &str) -> Option<usize> {
        self.quests.iter().position(|quest| {
            !quest.data.is_completed() && quest.data.name.eq_ignore_ascii_case(name)
        })
    }

    fn find_active_quest_mut(&mut self, name: &str) -> Option<&mut Quest> {
        self.quests
            .iter_mut()
            .find(|quest| !quest.data.is_completed() && quest.data.name.eq_ignore_ascii_case(name))
    }

    fn sort_quests(&mut self) {
        self.quests.sort_by_key(|quest| quest.data.is_completed());
    }

    pub fn set_quests(&mut self, quests: Vec<QuestData>) {
        let quests = quests
            .into_iter()
            .map(|data| self.new_quest(data))
            .collect();

        self.quests.set_items(quests);
        self.sort_quests();
    }

    pub fn set_quest(&mut self, data: QuestData) {
        match (
            self.find_active_quest_index(&data.name),
            data.is_completed(),
        ) {
            (Some(index), _) => {
                if let Some(quest) = self.quests.get_mut(index) {
                    quest.data = data;
                }
            }
            (None, false) => {
                let quest = self.new_quest(data);
                self.quests.push(quest);
            }
            (None, true) => return,
        }

        self.sort_quests();
    }

    pub fn set_quest_step(&mut self, name: String, current_step: u8) {
        let Some(quest) = self.find_active_quest_mut(&name) else {
            return;
        };

        quest.data.current_step = current_step;
    }

    pub fn set_quest_as_completed(&mut self, name: String, rewards: Vec<Item>) {
        let Some(quest) = self.find_active_quest_mut(&name) else {
            return;
        };

        quest.data.current_step = quest.data.max_step;
        quest.data.status = QuestStatus::Completed;

        self.inventory.extend(rewards);
        self.sort_quests();
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}
