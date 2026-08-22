use crate::collections::{Step, move_index};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameFocus {
    #[default]
    Input,
    RightPanel,
    NpcList,
    QuestList,
    RoomItemsList,
    InventoryGrid,
    ActionHistory,
}

impl GameFocus {
    pub const FOCUS_COUNT: usize = 7;

    const ORDER: [GameFocus; Self::FOCUS_COUNT] = [
        GameFocus::Input,
        GameFocus::NpcList,
        GameFocus::RoomItemsList,
        GameFocus::QuestList,
        GameFocus::ActionHistory,
        GameFocus::InventoryGrid,
        GameFocus::RightPanel,
    ];

    fn index(self) -> usize {
        match self {
            GameFocus::Input => 0,
            GameFocus::NpcList => 1,
            GameFocus::RoomItemsList => 2,
            GameFocus::QuestList => 3,
            GameFocus::ActionHistory => 4,
            GameFocus::InventoryGrid => 5,
            GameFocus::RightPanel => 6,
        }
    }

    pub fn next(&mut self) {
        *self = Self::ORDER[move_index(self.index(), Self::FOCUS_COUNT, Step::Next)]
    }

    pub fn prev(&mut self) {
        *self = Self::ORDER[move_index(self.index(), Self::FOCUS_COUNT, Step::Previous)]
    }
}
