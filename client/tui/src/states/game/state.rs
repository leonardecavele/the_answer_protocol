use crate::collections::BoundedLog;
use crate::data::manifest::Manifest;
use crate::states::game::interaction::{GameFocus, Overlays};
use crate::states::game::session::{
    ChatMessage, FightPhase, GroupState, PlayerState, Room, ServerState,
};
use crate::states::game::{Item, Npc};
use std::sync::Arc;

pub struct GameState {
    pub player: PlayerState,
    pub group: GroupState,
    pub room: Option<Room>,
    pub server: ServerState,
    pub overlays: Overlays,
    pub fight: FightPhase,
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
            room: None,
            server: ServerState::new(),
            overlays: Overlays::new(),
            fight: FightPhase::default(),
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

    pub fn find_npc(&self, id: &str) -> Option<&Npc> {
        self.room.as_ref()?.npcs.iter().find(|npc| npc.id == id)
    }

    pub fn find_item(&self, id: &str) -> Option<&Item> {
        self.room
            .as_ref()
            .and_then(|room| room.items.iter().find(|item| item.id == id))
            .or_else(|| self.player.inventory.iter().find(|item| item.id == id))
    }
}
