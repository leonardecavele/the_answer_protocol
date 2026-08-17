mod chat;
mod dialogue;
mod fight;
mod group;
mod log;
mod overlay;
mod player;
mod room;
mod server;

pub use chat::{ChatChannel, ChatMessage, ChatState};
pub use dialogue::{DialogueState, END_OF_DIALOGUE_TAG};
pub use fight::FightState;
pub use group::GroupState;
pub use overlay::{GameUiState, Overlay, OverlayKind};
pub use player::PlayerState;
pub use room::RoomState;
pub use server::ServerState;
pub use log::LogState;

use crate::data::manifest::Manifest;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameFocus {
    Input,
    RightPanel,
    NpcList,
    QuestList,
    RoomItemsList,
    InventoryGrid,
    ActionHistory,
}

impl Default for GameFocus {
    fn default() -> Self {
        GameFocus::Input
    }
}

pub struct GameState {
    pub player: PlayerState,
    pub group: GroupState,
    pub room: RoomState,
    pub server: ServerState,
    pub ui: GameUiState,
    pub fight: FightState,
    pub manifest: Arc<Manifest>,

    pub chat_history: ChatState,
    pub action_logs: LogState,

    pub current_focus: GameFocus,
}

impl GameState {
    pub fn new(manifest: Arc<Manifest>) -> Self {
        Self {
            player: PlayerState::new(),
            group: GroupState::new(),
            room: RoomState::new(),
            server: ServerState::new(),
            ui: GameUiState::new(),
            fight: FightState::new(),
            manifest,
            chat_history: ChatState::new(),
            action_logs: LogState::new(),
            current_focus: GameFocus::default(),
        }
    }

    pub fn log_action(&mut self, text: String) {
        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
        self.action_logs.push(format!("[{}] {}", time_str, text));
    }
}
