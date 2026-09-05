use super::{Overlay, OverlayPayload};
use crate::collections::SelectableList;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InvitationAction {
    Join,
    Cancel,
}

impl InvitationAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Join => "JOIN",
            Self::Cancel => "CANCEL",
        }
    }

    pub fn keyword(self) -> Option<&'static str> {
        match self {
            Self::Join => Some("GROUP JOIN"),
            Self::Cancel => None,
        }
    }
}

pub struct InvitationActionsState {
    pub leader: String,
    pub actions: SelectableList<InvitationAction>,
}

impl InvitationActionsState {
    pub fn new(leader: String) -> Self {
        let mut actions =
            SelectableList::with_items(vec![InvitationAction::Join, InvitationAction::Cancel]);
        actions.select_index(0);

        Self { leader, actions }
    }

    pub fn selected_command(&self) -> Option<&'static str> {
        self.actions.selected().and_then(|action| action.keyword())
    }
}

impl OverlayPayload for InvitationActionsState {
    fn extract(overlay: &Overlay) -> Option<&Self> {
        match overlay {
            Overlay::InvitationActions(state) => Some(state),
            _ => None,
        }
    }

    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self> {
        match overlay {
            Overlay::InvitationActions(state) => Some(state),
            _ => None,
        }
    }
}
