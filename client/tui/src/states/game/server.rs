pub struct ServerState {
    pub online_players_count: u32,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            online_players_count: 1,
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
