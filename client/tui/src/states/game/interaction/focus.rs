#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameFocus {
    Input,
    RightPanel,
    NpcList,
    QuestList,
    RoomItemsList,
    InventoryGrid,
    ActionHistory,
}

impl Default for GameFocus {
    fn default() -> Self {
        GameFocus::Input
    }
}
