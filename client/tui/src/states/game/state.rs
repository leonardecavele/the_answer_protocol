use crate::collections::BoundedLog;
use crate::data::manifest::Manifest;
use crate::states::game::interaction::{GameFocus, Overlays};
use crate::states::game::session::{
    ChatMessage, FightState, GroupState, PlayerState, RoomState, ServerState,
};
use crate::states::game::{Item, Npc};
use std::sync::Arc;

pub struct GameState {
    pub player: PlayerState,
    pub group: GroupState,
    pub room: RoomState,
    pub server: ServerState,
    pub overlays: Overlays,
    pub fight: FightState,
    pub manifest: Arc<Manifest>,

    pub chat_log: BoundedLog<ChatMessage>,
    pub action_log: BoundedLog<String>,

    pub focus: GameFocus,
}

// TODO: fermer le dialogue popup si le chef de groupe change de room
// TODO: si le game server est offline, il faut revenir sur la GameView

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
            chat_log: BoundedLog::with_max_capacity(200),
            action_log: BoundedLog::with_max_capacity(50),
            focus: GameFocus::default(),
        }
    }

    pub fn log_action(&mut self, text: String) {
        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
        self.action_log.push(format!("[{}] {}", time_str, text));
    }
}
