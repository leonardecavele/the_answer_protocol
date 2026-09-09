use super::{Overlay, OverlayPayload};
use crate::states::game::session::QuestId;

pub struct QuestDetailState {
    pub id: QuestId,
}

impl QuestDetailState {
    pub fn new(id: QuestId) -> Self {
        Self { id }
    }
}

impl OverlayPayload for QuestDetailState {
    fn extract(overlay: &Overlay) -> Option<&Self> {
        match overlay {
            Overlay::QuestDetail(state) => Some(state),
            _ => None,
        }
    }

    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self> {
        match overlay {
            Overlay::QuestDetail(state) => Some(state),
            _ => None,
        }
    }
}
