#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
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

