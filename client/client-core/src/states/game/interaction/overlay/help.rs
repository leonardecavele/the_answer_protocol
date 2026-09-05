use super::{Overlay, OverlayPayload};

pub struct HelpState;

impl OverlayPayload for HelpState {
    fn extract(overlay: &Overlay) -> Option<&Self> {
        match overlay {
            Overlay::Help(state) => Some(state),
            _ => None,
        }
    }

    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self> {
        match overlay {
            Overlay::Help(state) => Some(state),
            _ => None,
        }
    }
}
