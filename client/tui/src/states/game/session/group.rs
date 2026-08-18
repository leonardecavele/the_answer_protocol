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

    pub fn is_leader(&self, player_name: &str) -> bool {
        if let Some(leader) = &self.leader {
            leader == player_name
        } else {
            false
        }
    }

    pub fn leave(&mut self) {
        self.id = None;
        self.leader = None;
    }
}

impl Default for GroupState {
    fn default() -> Self {
        Self::new()
    }
}
