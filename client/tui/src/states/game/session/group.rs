use crate::collections::SelectableList;

pub struct GroupState {
    pub id: Option<String>,
    pub leader: Option<String>,
    pub invitations: SelectableList<String>,
}

impl GroupState {
    pub fn new() -> Self {
        Self {
            id: None,
            leader: None,
            invitations: SelectableList::new(),
        }
    }

    pub fn is_in_group(&self) -> bool {
        self.id.is_some()
    }

    pub fn is_leader(&self, player_name: Option<&str>) -> bool {
        if let Some(name) = player_name {
            return self
                .leader
                .as_deref()
                .is_some_and(|leader| leader.eq_ignore_ascii_case(name));
        }

        false
    }

    pub fn allows_move_by(&self, player_name: Option<&str>) -> bool {
        !self.is_in_group() || self.is_leader(player_name)
    }

    pub fn join(&mut self, id: String, leader: String) {
        self.id = Some(id);
        self.leader = Some(leader.clone());
        self.remove_invitation(&leader);
    }

    pub fn leave(&mut self) {
        self.id = None;
        self.leader = None;
    }

    pub fn invited_by(&mut self, leader: String) {
        if self.is_invited_by(&leader) {
            return;
        }

        self.invitations.push(leader);
    }

    pub fn remove_invitation(&mut self, leader: &str) {
        self.invitations
            .retain(|invitation| !invitation.eq_ignore_ascii_case(leader));
    }

    fn is_invited_by(&self, leader: &str) -> bool {
        self.invitations
            .iter()
            .any(|invitation| invitation.eq_ignore_ascii_case(leader))
    }
}

impl Default for GroupState {
    fn default() -> Self {
        Self::new()
    }
}
