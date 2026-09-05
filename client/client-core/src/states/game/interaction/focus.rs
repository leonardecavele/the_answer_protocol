use crate::collections::{Step, move_index};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameFocus {
    #[default]
    Input,
    RightPanel,
    PlayerList,
    NpcList,
    QuestList,
    InvitationList,
    RoomItemsList,
    InventoryGrid,
    ActionHistory,
}

impl GameFocus {
    pub const FOCUS_COUNT: usize = 9;

    const ORDER: [GameFocus; Self::FOCUS_COUNT] = [
        GameFocus::Input,
        GameFocus::InvitationList,
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
            GameFocus::InvitationList => 1,
            GameFocus::PlayerList => 2,
            GameFocus::NpcList => 3,
            GameFocus::RoomItemsList => 4,
            GameFocus::QuestList => 5,
            GameFocus::ActionHistory => 6,
            GameFocus::InventoryGrid => 7,
            GameFocus::RightPanel => 8,
        }
    }

    pub fn next(&mut self) {
        *self = Self::ORDER[move_index(self.index(), Self::FOCUS_COUNT, Step::Next)]
    }

    pub fn prev(&mut self) {
        *self = Self::ORDER[move_index(self.index(), Self::FOCUS_COUNT, Step::Previous)]
    }
}
