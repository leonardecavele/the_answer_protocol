#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
