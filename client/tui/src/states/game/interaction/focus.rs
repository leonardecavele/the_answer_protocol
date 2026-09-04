use crate::collections::{Step, move_index};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameFocus {
    #[default]
    Input,
    RightPanel,
    PlayerList,
    NpcList,
    QuestList,
    RoomItemsList,
    InventoryGrid,
    ActionHistory,
}

impl GameFocus {
    pub const FOCUS_COUNT: usize = 8;

    const ORDER: [GameFocus; Self::FOCUS_COUNT] = [
        GameFocus::Input,
        GameFocus::PlayerList,
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
            GameFocus::PlayerList => 1,
            GameFocus::NpcList => 2,
            GameFocus::RoomItemsList => 3,
            GameFocus::QuestList => 4,
            GameFocus::ActionHistory => 5,
            GameFocus::InventoryGrid => 6,
            GameFocus::RightPanel => 7,
        }
    }

    pub fn next(&mut self) {
        *self = Self::ORDER[move_index(self.index(), Self::FOCUS_COUNT, Step::Next)]
    }

    pub fn prev(&mut self) {
        *self = Self::ORDER[move_index(self.index(), Self::FOCUS_COUNT, Step::Previous)]
    }
}
