use crate::collections::BoundedLog;
use crate::data::manifest::Manifest;
use crate::states::game::interaction::{DialogueState, GameFocus, Overlay, OverlayKind, Overlays};
use crate::states::game::session::{
    ChatMessage, FightState, GroupState, PlayerState, Room, ServerState,
};
use crate::states::game::{Item, Npc};
use client_core::Assets;
use std::sync::Arc;
use std::time::{Duration, Instant};

const DIALOGUE_REOPEN_COOLDOWN: Duration = Duration::from_millis(300);

pub struct GameState {
    pub player: PlayerState,
    pub group: GroupState,
    pub room: Option<Room>,
    pub server: ServerState,
    pub overlays: Overlays,
    pub fight: FightState,
    pub manifest: Arc<Manifest>,
    pub assets: Assets,

    pub chat_log: BoundedLog<ChatMessage>,
    pub action_log: BoundedLog<String>,

    pub inspected_npc: Option<String>,
    focus: GameFocus,
    dialogue_closed_at: Option<Instant>,
}

impl GameState {
    pub fn new(manifest: Arc<Manifest>, assets: Assets) -> Self {
        Self {
            player: PlayerState::new(),
            group: GroupState::new(),
            room: None,
            server: ServerState::new(),
            overlays: Overlays::new(),
            fight: FightState::default(),
            manifest,
            assets,
            chat_log: BoundedLog::with_max_capacity(200),
            action_log: BoundedLog::with_max_capacity(50),
            focus: GameFocus::default(),
            inspected_npc: None,
            dialogue_closed_at: None,
        }
    }

    pub fn focus(&self) -> GameFocus {
        self.focus
    }

    pub fn set_focus(&mut self, focus: GameFocus) {
        if self.focus == focus {
            return;
        }

        self.focus = focus;
        self.clear_selections();
    }

    pub fn focus_next(&mut self) {
        let mut focus = self.focus;
        focus.next();
        self.set_focus(focus);
    }

    pub fn focus_prev(&mut self) {
        let mut focus = self.focus;
        focus.prev();
        self.set_focus(focus);
    }

    pub fn clear_selections(&mut self) {
        if let Some(room) = self.room.as_mut() {
            room.npcs.clear_selection();
            room.items.clear_selection();
        }

        self.player.inventory.clear_selection();
        self.player.quests.clear_selection();
        self.group.invitations.clear_selection();
    }

    pub fn open_dialogue(&mut self, dialogue: DialogueState) {
        self.overlays.open(Overlay::Dialogue(dialogue));
    }

    pub fn close_top_overlay(&mut self) {
        let Some(kind) = self.overlays.top_kind() else {
            return;
        };

        match kind {
            OverlayKind::Dialogue => self.close_dialogue(),
            OverlayKind::NpcActions
            | OverlayKind::ItemActions
            | OverlayKind::QuestDetail
            | OverlayKind::PlayerActions
            | OverlayKind::InvitationActions => {
                self.overlays.close_top();
                self.clear_selections();
            }
            _ => self.overlays.close_top(),
        }
    }

    pub fn close_dialogue(&mut self) {
        self.overlays.close::<DialogueState>();
        self.inspected_npc = None;
        self.dialogue_closed_at = Some(Instant::now());
    }

    pub fn close_all_overlays(&mut self) {
        self.overlays.close_all();
        self.inspected_npc = None;
    }

    pub fn end_npc_interaction(&mut self) {
        if self.overlays.is_open::<DialogueState>() {
            self.close_dialogue();
        } else {
            self.inspected_npc = None;
        }
    }

    pub fn dialogue_cooldown_elapsed(&self) -> bool {
        self.dialogue_closed_at
            .is_none_or(|closed_at| closed_at.elapsed() >= DIALOGUE_REOPEN_COOLDOWN)
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
