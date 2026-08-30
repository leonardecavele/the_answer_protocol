use super::{Overlay, OverlayPayload};

pub struct NpcActionsState {
    pub npc_id: String,
}

impl NpcActionsState {
    pub fn new(npc_id: String) -> Self {
        Self { npc_id }
    }
}

impl OverlayPayload for NpcActionsState {
    fn extract(overlay: &Overlay) -> Option<&Self> {
        match overlay {
            Overlay::NpcActions(state) => Some(state),
            _ => None,
        }
    }

    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self> {
        match overlay {
            Overlay::NpcActions(state) => Some(state),
            _ => None,
        }
    }
}
