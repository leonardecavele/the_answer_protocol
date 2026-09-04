use super::{Overlay, OverlayPayload};

pub const END_OF_DIALOGUE_TAG: &str = "[end of dialogue]";

#[derive(Debug, Clone)]
pub struct DialogueState {
    pub npc_id: String,
    pub npc_name: String,
    pub full_text: String,
    pub ends_dialog: bool,
}

impl DialogueState {
    pub fn new(npc_id: String, npc_name: String, full_text: String, ends_dialog: bool) -> Self {
        Self {
            npc_id,
            npc_name,
            full_text,
            ends_dialog,
        }
    }

    pub fn add(&mut self, text: String, ends_dialog: bool) {
        self.full_text.push_str("\n\n");
        self.full_text.push_str(&text);
        self.ends_dialog = ends_dialog;
    }

    pub fn char_count(&self) -> usize {
        self.full_text.chars().count()
    }
}

impl OverlayPayload for DialogueState {
    fn extract(overlay: &Overlay) -> Option<&Self> {
        match overlay {
            Overlay::Dialogue(state) => Some(state),
            _ => None,
        }
    }

    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self> {
        match overlay {
            Overlay::Dialogue(state) => Some(state),
            _ => None,
        }
    }
}
