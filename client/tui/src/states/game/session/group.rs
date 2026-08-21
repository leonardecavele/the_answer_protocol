pub struct GroupState {
    pub id: Option<String>,
    pub leader: Option<String>,
}

impl GroupState {
    pub fn new() -> Self {
        Self {
            id: None,
            leader: None,
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
        self.leader = Some(leader.to_uppercase());
    }

    pub fn leave(&mut self) {
        *self = Self::default()
    }
}

impl Default for GroupState {
    fn default() -> Self {
        Self::new()
    }
}
