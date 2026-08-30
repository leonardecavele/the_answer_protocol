use super::{Overlay, OverlayPayload};
use crate::collections::SelectableList;
use crate::data::manifest::NpcKind;

pub struct NpcActionsState {
    pub npc_id: String,
    pub actions: SelectableList<String>,
}

impl NpcActionsState {
    pub const TALK: &'static str = "TALK";
    pub const ATTACK: &'static str = "ATTACK";
    pub const QUEST: &'static str = "QUEST";
    pub const CANCEL: &'static str = "CANCEL";

    pub fn new(npc_id: String, kind: &NpcKind) -> Self {
        let mut actions: Vec<String> = match kind {
            NpcKind::Enemy => vec![Self::TALK, Self::ATTACK],
            NpcKind::QuestGiver => vec![Self::TALK, Self::QUEST],
            NpcKind::Dialogue | NpcKind::Normal => vec![Self::TALK],
        }
        .into_iter()
        .map(str::to_string)
        .collect();

        actions.push(Self::CANCEL.to_string());

        Self {
            npc_id,
            actions: SelectableList::with_items(actions),
        }
    }

    pub fn selected_command(&self) -> Option<&str> {
        self.actions
            .selected()
            .map(String::as_str)
            .filter(|action| *action != Self::CANCEL)
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
