use super::{Overlay, OverlayPayload};
use crate::collections::SelectableList;
use client_api::ApiRequest;
use client_api::commands::GroupJoinCommand;

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

    pub fn request(self, leader: &str) -> Option<ApiRequest> {
        match self {
            Self::Join => Some(ApiRequest::GroupJoin(GroupJoinCommand {
                leader_name: leader.to_string(),
            })),
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

    pub fn selected_request(&self) -> Option<ApiRequest> {
        self.actions
            .selected()
            .and_then(|action| action.request(&self.leader))
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
