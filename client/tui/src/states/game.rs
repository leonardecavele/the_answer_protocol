pub struct GameState {
    pub player_name: Option<String>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player_name: None,
        }
    }
}
