use crate::states::game::DialogueState;
use std::time::{Duration, Instant};

pub enum Overlay {
    Help,
    Chat,
    NpcActions { npc_id: String },
    ItemActions { item_id: String },
    ItemDetail { item_id: String },
    QuestDetail { quest_id: String },
    Dialogue(DialogueState),
}

#[derive(Clone, Copy, PartialEq)]
pub enum OverlayKind {
    Help,
    Chat,
    NpcActions,
    ItemActions,
    ItemDetail,
    QuestDetail,
    Dialogue,
}

impl Overlay {
    pub fn kind(&self) -> OverlayKind {
        match self {
            Overlay::Help => OverlayKind::Help,
            Overlay::Chat => OverlayKind::Chat,
            Overlay::NpcActions { .. } => OverlayKind::NpcActions,
            Overlay::ItemActions { .. } => OverlayKind::ItemActions,
            Overlay::ItemDetail { .. } => OverlayKind::ItemDetail,
            Overlay::QuestDetail { .. } => OverlayKind::QuestDetail,
            Overlay::Dialogue(_) => OverlayKind::Dialogue,
        }
    }
}

impl OverlayKind {
    pub fn is_modal(&self) -> bool {
        !matches!(self, OverlayKind::Chat)
    }
}

pub struct Overlays {
    pub inspected_entity: Option<String>,
    dialogue_closed_at: Option<Instant>,
    overlays: Vec<Overlay>,
}

impl Overlays {
    pub fn new() -> Self {
        Self {
            inspected_entity: None,
            dialogue_closed_at: None,
            overlays: Vec::new(),
        }
    }

    pub fn open(&mut self, overlay: Overlay) {
        let kind = overlay.kind();
        self.overlays.retain(|o| o.kind() != kind);
        self.overlays.push(overlay);
    }

    pub fn toggle(&mut self, overlay: Overlay) {
        let kind = overlay.kind();
        if self.is_open(kind) {
            self.close(kind);
        } else {
            self.open(overlay);
        }
    }

    pub fn close_top(&mut self) {
        if let Some(overlay) = self.overlays.pop() {
            self.after_close(overlay.kind());
        }
    }

    pub fn close(&mut self, kind: OverlayKind) {
        self.overlays.retain(|o| o.kind() != kind);
        self.after_close(kind);
    }

    pub fn close_all(&mut self) {
        let had_dialogue = self.is_open(OverlayKind::Dialogue);
        self.overlays.clear();
        if had_dialogue {
            self.after_close(OverlayKind::Dialogue);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Overlay> {
        self.overlays.iter()
    }

    pub fn top_kind(&self) -> Option<OverlayKind> {
        self.overlays.last().map(Overlay::kind)
    }

    pub fn is_open(&self, kind: OverlayKind) -> bool {
        self.overlays.iter().any(|o| o.kind() == kind)
    }

    pub fn target_of(&self, kind: OverlayKind) -> Option<&str> {
        self.overlays
            .iter()
            .rev()
            .find(|o| o.kind() == kind)
            .and_then(|o| match o {
                Overlay::NpcActions { npc_id } => Some(npc_id.as_str()),
                Overlay::ItemActions { item_id } | Overlay::ItemDetail { item_id } => {
                    Some(item_id.as_str())
                }
                Overlay::QuestDetail { quest_id } => Some(quest_id.as_str()),
                _ => None,
            })
    }

    pub fn dialogue(&self) -> Option<&DialogueState> {
        self.overlays.iter().rev().find_map(|o| match o {
            Overlay::Dialogue(dialogue) => Some(dialogue),
            _ => None,
        })
    }

    pub fn dialogue_mut(&mut self) -> Option<&mut DialogueState> {
        self.overlays.iter_mut().rev().find_map(|o| match o {
            Overlay::Dialogue(dialogue) => Some(dialogue),
            _ => None,
        })
    }

    pub fn dialogue_cooldown_elapsed(&self) -> bool {
        if let Some(time) = self.dialogue_closed_at {
            if time.elapsed() < Duration::from_millis(300) {
                return false;
            }
        }
        true
    }

    fn after_close(&mut self, kind: OverlayKind) {
        if kind == OverlayKind::Dialogue {
            self.inspected_entity = None;
            self.dialogue_closed_at = Some(Instant::now());
        }
    }
}

impl Default for Overlays {
    fn default() -> Self {
        Self::new()
    }
}
