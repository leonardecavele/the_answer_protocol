use super::{Overlay, OverlayPayload};

pub struct ItemDetailState {
    pub item_id: String,
}

impl ItemDetailState {
    pub fn new(item_id: String) -> Self {
        Self { item_id }
    }
}

impl OverlayPayload for ItemDetailState {
    fn extract(overlay: &Overlay) -> Option<&Self> {
        match overlay {
            Overlay::ItemDetail(state) => Some(state),
            _ => None,
        }
    }

    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self> {
        match overlay {
            Overlay::ItemDetail(state) => Some(state),
            _ => None,
        }
    }
}
