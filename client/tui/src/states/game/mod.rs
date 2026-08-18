mod chat_log;
mod dialogue;
mod fight;
mod group;
mod action_log;
mod overlays;
mod player;
mod room;
mod server;

pub use chat_log::{ChatChannel, ChatMessage, ChatLog};
pub use dialogue::{DialogueState, END_OF_DIALOGUE_TAG};
pub use fight::FightState;
pub use group::GroupState;
pub use overlays::{Overlays, Overlay, OverlayKind};
pub use player::PlayerState;
pub use room::RoomState;
pub use server::ServerState;
pub use action_log::ActionLog;

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
    pub overlays: Overlays,
    pub fight: FightState,
    pub manifest: Arc<Manifest>,

    pub chat_log: ChatLog,
    pub action_log: ActionLog,

    pub focus: GameFocus,
}

impl GameState {
    pub fn new(manifest: Arc<Manifest>) -> Self {
        Self {
            player: PlayerState::new(),
            group: GroupState::new(),
            room: RoomState::new(),
            server: ServerState::new(),
            overlays: Overlays::new(),
            fight: FightState::new(),
            manifest,
            chat_log: ChatLog::new(),
            action_log: ActionLog::new(),
            focus: GameFocus::default(),
        }
    }

    pub fn log_action(&mut self, text: String) {
        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
        self.action_log.push(format!("[{}] {}", time_str, text));
    }
}
