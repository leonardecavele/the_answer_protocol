use super::{Overlay, OverlayPayload};

pub struct ChatState;

impl OverlayPayload for ChatState {
    fn extract(overlay: &Overlay) -> Option<&Self> {
        match overlay {
            Overlay::Chat(state) => Some(state),
            _ => None,
        }
    }

    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self> {
        match overlay {
            Overlay::Chat(state) => Some(state),
            _ => None,
        }
    }
}
