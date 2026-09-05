use super::{Overlay, OverlayPayload};
use crate::collections::SelectableList;
use crate::data::manifest::NpcKind;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NpcAction {
    Talk,
    Attack,
    Fight,
    Quest,
    Cancel,
}

impl NpcAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Talk => "TALK",
            Self::Attack => "ATTACK",
            Self::Fight => "DUEL",
            Self::Quest => "QUEST",
            Self::Cancel => "CANCEL",
        }
    }

    pub fn keyword(self) -> Option<&'static str> {
        match self {
            Self::Talk => Some("TALK"),
            Self::Attack => Some("ATTACK"),
            Self::Fight => Some("FC"),
            Self::Quest => Some("QUEST"),
            Self::Cancel => None,
        }
    }
}

pub struct NpcActionsState {
    pub npc_id: String,
    pub actions: SelectableList<NpcAction>,
}

impl NpcActionsState {
    pub fn new(npc_id: String, kind: &NpcKind) -> Self {
        let mut actions = match kind {
            NpcKind::Enemy => vec![NpcAction::Talk, NpcAction::Attack, NpcAction::Fight],
            NpcKind::QuestGiver => vec![NpcAction::Talk, NpcAction::Quest],
            NpcKind::Dialogue | NpcKind::Normal => vec![NpcAction::Talk],
        };

        actions.push(NpcAction::Cancel);

        let mut actions = SelectableList::with_items(actions);
        actions.select_index(0);

        Self { npc_id, actions }
    }

    pub fn selected_command(&self) -> Option<&'static str> {
        self.actions.selected().and_then(|action| action.keyword())
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
