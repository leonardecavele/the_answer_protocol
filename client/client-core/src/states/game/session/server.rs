pub struct ServerState {
    pub online_players_count: u32,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            online_players_count: 1,
        }
    }

    pub fn set_online_count(&mut self, count: u32) {
        self.online_players_count = count;
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
