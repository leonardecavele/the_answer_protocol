use super::{Overlay, OverlayPayload};
use crate::collections::SelectableList;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    Invite,
    Cancel,
}

impl PlayerAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Invite => "INVITE",
            Self::Cancel => "CANCEL",
        }
    }

    pub fn keyword(self) -> Option<&'static str> {
        match self {
            Self::Invite => Some("GROUP INVITE"),
            Self::Cancel => None,
        }
    }
}

pub struct PlayerActionsState {
    pub player_name: String,
    pub actions: SelectableList<PlayerAction>,
}

impl PlayerActionsState {
    pub fn new(player_name: String, can_invite: bool) -> Self {
        let mut actions = Vec::new();

        if can_invite {
            actions.push(PlayerAction::Invite);
        }

        actions.push(PlayerAction::Cancel);

        let mut actions = SelectableList::with_items(actions);
        actions.select_index(0);

        Self {
            player_name,
            actions,
        }
    }

    pub fn selected_command(&self) -> Option<&'static str> {
        self.actions.selected().and_then(|action| action.keyword())
    }
}

impl OverlayPayload for PlayerActionsState {
    fn extract(overlay: &Overlay) -> Option<&Self> {
        match overlay {
            Overlay::PlayerActions(state) => Some(state),
            _ => None,
        }
    }

    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self> {
        match overlay {
            Overlay::PlayerActions(state) => Some(state),
            _ => None,
        }
    }
}
