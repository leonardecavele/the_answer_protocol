use super::{Overlay, OverlayPayload};

pub struct QuestDetailState {
    pub name: String,
}

impl QuestDetailState {
    pub fn new(name: String) -> Self {
        Self { name }
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
