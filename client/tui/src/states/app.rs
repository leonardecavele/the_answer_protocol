use crate::data::manifest::Manifest;
use crate::states::game::GameState;
use crate::states::network::NetworkState;
use crate::states::ui::UiState;

pub struct AppState {
    pub should_quit: bool,
    pub ui: UiState,
    pub network: NetworkState,
    pub game: GameState,
}

impl AppState {
    pub fn new(ip: String, port: String, manifest: Manifest) -> Self {
        Self {
            should_quit: false,
            ui: UiState::new(),
            network: NetworkState::new(ip, port),
            game: GameState::new(manifest),
        }
    }
}
