use super::{Overlay, OverlayPayload};
use crate::collections::SelectableList;
use client_api::ApiRequest;
use client_api::commands::GroupInviteCommand;

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

    pub fn request(self, player_name: &str) -> Option<ApiRequest> {
        match self {
            Self::Invite => Some(ApiRequest::GroupInvite(GroupInviteCommand {
                username: player_name.to_string(),
            })),
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

    pub fn selected_request(&self) -> Option<ApiRequest> {
        self.actions
            .selected()
            .and_then(|action| action.request(&self.player_name))
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
