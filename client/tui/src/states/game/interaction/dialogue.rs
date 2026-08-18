use std::time::Instant;

pub const END_OF_DIALOGUE_TAG: &str = "[end of dialogue]";

#[derive(Debug, Clone)]
pub struct DialogueState {
    pub npc_id: String,
    pub npc_name: String,
    pub full_text: String,
    pub visible_chars: usize,
    pub ends_dialog: bool,
    pub last_tick: Instant,
}

impl DialogueState {
    pub fn new(npc_id: String, npc_name: String, full_text: String, ends_dialog: bool) -> Self {
        Self {
            npc_id,
            npc_name,
            full_text,
            visible_chars: 0,
            ends_dialog,
            last_tick: Instant::now(),
        }
    }

    pub fn add(&mut self, text: String, ends_dialog: bool) {
        self.full_text.push_str("\n\n");
        self.full_text.push_str(&text);
        self.ends_dialog = ends_dialog;
        self.visible_chars += 2;
    }
}
