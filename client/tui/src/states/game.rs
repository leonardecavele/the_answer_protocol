use crate::data::manifest::Manifest;
use api_client::commands::QuestData;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

//region Dialogue
pub const END_OF_DIALOGUE_TAG: &str = "[end of dialogue]";

#[derive(Debug, Clone)]
pub struct DialogueState {
    pub npc_id: String,
    pub npc_name: String,
    pub full_text: String,
    pub visible_chars: usize,
    pub ends_dialog: bool,
    pub last_tick: Instant,
}

impl DialogueState {
    pub fn new(npc_id: String, npc_name: String, full_text: String, ends_dialog: bool) -> Self {
        Self {
            npc_id,
            npc_name,
            full_text,
            visible_chars: 0,
            ends_dialog,
            last_tick: Instant::now(),
        }
    }

    pub fn add(&mut self, text: String, ends_dialog: bool) {
        self.full_text.push_str("\n\n");
        self.full_text.push_str(&text);
        self.ends_dialog = ends_dialog;
        self.visible_chars += 2;
    }
}
//endregion

//region Chat
#[derive(Debug, Clone)]
pub enum ChatChannel {
    Global,
    Private(String),
    Room,
    Group,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub channel: ChatChannel,
    pub sender: String,
    pub content: String,
}
//endregion

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

//region Sub-states
pub struct PlayerState {
    pub name: Option<String>,
    pub hp: u32,
    pub max_hp: u32,
    pub inventory: Vec<String>,
    pub quests: Vec<QuestData>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            name: None,
            hp: 100,
            max_hp: 100,
            inventory: Vec::new(),
            quests: Vec::new(),
        }
    }

    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.hp = self.hp.saturating_sub(amount);
    }

    pub fn is_dead(&self) -> bool {
        self.hp == 0
    }
}

pub struct GroupState {
    pub id: Option<String>,
    pub leader: Option<String>,
}

impl GroupState {
    pub fn new() -> Self {
        Self {
            id: None,
            leader: None,
        }
    }

    pub fn is_in_group(&self) -> bool {
        self.id.is_some()
    }

    pub fn is_leader(&self, player_name: &str) -> bool {
        if let Some(leader) = &self.leader {
            leader == player_name
        } else {
            false
        }
    }

    pub fn leave(&mut self) {
        self.id = None;
        self.leader = None;
    }
}

pub struct Npc {
    pub id: String,
    pub is_alive: bool,
}

pub struct RoomState {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub exits: HashMap<String, String>,
    pub players: Vec<String>,
    pub npcs: Vec<Npc>,
    pub items: Vec<String>,
}

impl RoomState {
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            description: None,
            exits: HashMap::new(),
            players: Vec::new(),
            npcs: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.id = None;
        self.name = None;
        self.description = None;
        self.exits.clear();
        self.players.clear();
        self.npcs.clear();
        self.items.clear();
    }

    pub fn has_exit(&self, direction: &str) -> bool {
        self.exits.contains_key(direction)
    }
}

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

pub struct GameUiState {
    pub current_focus: GameFocus,
    pub active_npc_popup: Option<String>,
    pub active_item_popup: Option<String>,
    pub active_item_view_popup: Option<String>,
    pub show_help_overlay: bool,
    pub show_chat: bool,
    pub inventory_cursor: usize,
    pub focused_entity_id: Option<String>,
    pub active_dialogue: Option<DialogueState>,
    pub dialogue_closed_at: Option<Instant>,
}

impl GameUiState {
    pub fn new() -> Self {
        Self {
            current_focus: GameFocus::default(),
            active_npc_popup: None,
            active_item_popup: None,
            active_item_view_popup: None,
            show_help_overlay: false,
            show_chat: false,
            inventory_cursor: 0,
            focused_entity_id: None,
            active_dialogue: None,
            dialogue_closed_at: None,
        }
    }

    pub fn is_npc_dialogue_available(&self) -> bool {
        if let Some(time) = self.dialogue_closed_at {
            if time.elapsed() < Duration::from_millis(300) {
                return false;
            }
        }
        true
    }

    pub fn close_dialogue(&mut self) {
        self.active_dialogue = None;
        self.focused_entity_id = None;
        self.dialogue_closed_at = Some(Instant::now());
    }

    pub fn close_all_popups(&mut self) {
        self.active_npc_popup = None;
        self.active_item_popup = None;
        self.active_item_view_popup = None;
        self.close_dialogue();
    }
}
//endregion

pub struct GameState {
    pub player: PlayerState,
    pub group: GroupState,
    pub room: RoomState,
    pub server: ServerState,
    pub ui: GameUiState,
    pub manifest: Arc<Manifest>,

    pub chat_history: Vec<ChatMessage>,
    pub action_logs: Vec<String>,
}

impl GameState {
    pub fn new(manifest: Arc<Manifest>) -> Self {
        Self {
            player: PlayerState::new(),
            group: GroupState::new(),
            room: RoomState::new(),
            server: ServerState::new(),
            ui: GameUiState::new(),
            manifest,
            chat_history: Vec::new(),
            action_logs: Vec::new(),
        }
    }

    pub fn log_action(&mut self, text: String) {
        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
        self.action_logs.push(format!("[{}] {}", time_str, text));
    }
}
