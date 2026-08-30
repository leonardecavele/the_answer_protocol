use crate::states::game::interaction::overlay::{Overlay, OverlayPayload};
use crate::states::game::{DialogueState, OverlayKind};
use std::time::{Duration, Instant};

pub struct Overlays {
    pub inspected_npc: Option<String>,
    dialogue_closed_at: Option<Instant>,
    overlays: Vec<Overlay>,
}

impl Default for Overlays {
    fn default() -> Self {
        Self::new()
    }
}

impl Overlays {
    pub fn new() -> Self {
        Self {
            inspected_npc: None,
            dialogue_closed_at: None,
            overlays: Vec::new(),
        }
    }

    pub fn get<T: OverlayPayload>(&self) -> Option<&T> {
        self.overlays.iter().rev().find_map(T::extract)
    }

    pub fn get_mut<T: OverlayPayload>(&mut self) -> Option<&mut T> {
        self.overlays.iter_mut().rev().find_map(T::extract_mut)
    }

    pub fn open(&mut self, overlay: Overlay) {
        let discriminant = overlay.discriminant();
        self.overlays.retain(|o| o.discriminant() != discriminant);
        self.overlays.push(overlay);
    }

    fn after_dialogue_closed(&mut self) {
        // TODO: modifier ce comportement qui n'as pas sa place ici
        self.inspected_npc = None;
        self.dialogue_closed_at = Some(Instant::now());
    }

    pub fn open_dialogue(&mut self, dialogue: DialogueState) {
        self.open(Overlay::Dialogue(dialogue));
    }

    pub fn toggle(&mut self, overlay: Overlay) {
        let discriminant = overlay.discriminant();

        if self
            .overlays
            .iter()
            .any(|o| o.discriminant() == discriminant)
        {
            self.overlays.retain(|o| o.discriminant() != discriminant);
        } else {
            self.open(overlay);
        }
    }

    pub fn close<T: OverlayPayload>(&mut self) {
        self.overlays.retain(|o| T::extract(o).is_none());
    }

    pub fn close_dialogue(&mut self) {
        self.close::<DialogueState>();
        self.after_dialogue_closed();
    }

    pub fn close_top(&mut self) {
        self.overlays.pop();
    }

    pub fn close_all(&mut self) {
        let had_dialogue = self.is_open::<DialogueState>();
        self.overlays.clear();

        if had_dialogue {
            self.after_dialogue_closed();
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Overlay> {
        self.overlays.iter()
    }

    pub fn top_kind(&self) -> Option<OverlayKind> {
        self.overlays.last().map(Overlay::kind)
    }

    pub fn is_open<T: OverlayPayload>(&self) -> bool {
        self.get::<T>().is_some()
    }

    pub fn dialogue_cooldown_elapsed(&self) -> bool {
        if let Some(time) = self.dialogue_closed_at
            && time.elapsed() < Duration::from_millis(300)
        {
            return false;
        }
        true
    }
}
