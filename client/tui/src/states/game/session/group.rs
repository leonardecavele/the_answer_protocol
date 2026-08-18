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
        self.leader.is_some() && self.leader.as_deref() == player_name
    }

    pub fn allows_move_by(&self, player_name: Option<&str>) -> bool {
        !self.is_in_group() || self.is_leader(player_name)
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
