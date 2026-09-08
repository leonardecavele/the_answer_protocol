use client_api::commands::QuestData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuestId(u64);

pub struct Quest {
    pub id: QuestId,
    pub data: QuestData,
}

impl Quest {
    pub(super) fn new(id: u64, data: QuestData) -> Self {
        Self {
            id: QuestId(id),
            data,
        }
    }
}
