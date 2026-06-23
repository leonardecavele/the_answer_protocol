use crate::app::App;
use crate::events::GameEvent;
use tracing::info;

impl App {
    pub(crate) fn handle_game_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::PlayerJoined { player_name } => info!("Player {} joined", player_name),
            _ => {}
        }
    }
}
