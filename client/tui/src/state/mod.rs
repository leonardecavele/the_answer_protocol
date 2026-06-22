pub mod network;
pub mod game;
pub mod ui;

pub use network::{NetworkState, ConnectionState};
pub use game::{GameState, ChatScope, ChatEntry};
pub use ui::{UiState, NotificationType, Notification, GameFocus};

pub struct AppState {
    pub net: NetworkState,
    pub game: GameState,
    pub ui: UiState,
    pub assets: crate::assets::AssetManager,
    pub registry: std::sync::Arc<crate::commands::CommandRegistry>,
    pub should_quit: bool,
}

impl AppState {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            net: NetworkState::new(ip, port),
            game: GameState::new(),
            ui: UiState::new(),
            assets: crate::assets::AssetManager::new(),
            registry: std::sync::Arc::new(crate::commands::CommandRegistry::new()),
            should_quit: false,
        }
    }
}
