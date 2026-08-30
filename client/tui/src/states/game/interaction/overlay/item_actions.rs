use super::{Overlay, OverlayPayload};

pub struct ItemActionsState {
    pub item_id: String,
}

impl ItemActionsState {
    pub fn new(item_id: String) -> Self {
        Self { item_id }
    }
}

impl OverlayPayload for ItemActionsState {
    fn extract(overlay: &Overlay) -> Option<&Self> {
        match overlay {
            Overlay::ItemActions(state) => Some(state),
            _ => None,
        }
    }

    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self> {
        match overlay {
            Overlay::ItemActions(state) => Some(state),
            _ => None,
        }
    }
}
