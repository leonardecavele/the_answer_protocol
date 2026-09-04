mod chat;
mod dialogue;
mod help;
mod item_actions;
mod item_detail;
mod npc_actions;
mod player_actions;
mod quest_detail;

pub use chat::ChatState;
pub use dialogue::{DialogueState, END_OF_DIALOGUE_TAG};
pub use help::HelpState;
pub use item_actions::{ItemActionsState, ItemLocation};
pub use item_detail::ItemDetailState;
pub use npc_actions::{NpcAction, NpcActionsState};
pub use player_actions::{PlayerAction, PlayerActionsState};
pub use quest_detail::QuestDetailState;

use std::mem;
use std::mem::Discriminant;

pub enum Overlay {
    Help(HelpState),
    Chat(ChatState),
    NpcActions(NpcActionsState),
    ItemActions(ItemActionsState),
    ItemDetail(ItemDetailState),
    QuestDetail(QuestDetailState),
    PlayerActions(PlayerActionsState),
    Dialogue(DialogueState),
}

#[derive(Clone, Copy, PartialEq)]
pub enum OverlayKind {
    Help,
    Chat,
    NpcActions,
    ItemActions,
    ItemDetail,
    QuestDetail,
    PlayerActions,
    Dialogue,
}

impl OverlayKind {
    pub fn is_modal(&self) -> bool {
        !matches!(self, OverlayKind::Chat)
    }
}

impl Overlay {
    pub fn discriminant(&self) -> Discriminant<Self> {
        mem::discriminant(self)
    }

    pub fn kind(&self) -> OverlayKind {
        match self {
            Overlay::Help(_) => OverlayKind::Help,
            Overlay::Chat(_) => OverlayKind::Chat,
            Overlay::NpcActions(_) => OverlayKind::NpcActions,
            Overlay::ItemActions(_) => OverlayKind::ItemActions,
            Overlay::ItemDetail(_) => OverlayKind::ItemDetail,
            Overlay::QuestDetail(_) => OverlayKind::QuestDetail,
            Overlay::PlayerActions(_) => OverlayKind::PlayerActions,
            Overlay::Dialogue(_) => OverlayKind::Dialogue,
        }
    }
}

pub trait OverlayPayload: Sized {
    fn extract(overlay: &Overlay) -> Option<&Self>;
    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self>;
}
