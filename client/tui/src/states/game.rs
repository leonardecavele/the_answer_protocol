use crate::data::manifest::Manifest;
use api_client::protocol::command::resource_interaction::quests::QuestListEntry;
use std::collections::HashMap;
use std::time::{Duration, Instant};

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

pub struct GameState {
    pub player_name: Option<String>,
    pub chat_history: Vec<ChatMessage>,
    pub room_players: Vec<String>,
    pub group_id: Option<String>,
    pub group_leader: Option<String>,
    pub online_players_count: u32,
    pub hp: u32,
    pub max_hp: u32,
    pub manifest: Manifest,
    pub room_npcs: Vec<String>,
    pub current_room_items: Vec<String>,
    pub inventory: Vec<String>,
    pub quests: Vec<QuestListEntry>,
    pub current_room_id: Option<String>,
    pub current_room_name: Option<String>,
    pub current_room_description: Option<String>,
    pub current_room_exits: HashMap<String, String>,
    pub focused_entity_id: Option<String>,
    pub action_logs: Vec<String>,
    pub active_dialogue: Option<DialogueState>,
    pub dialogue_closed_at: Option<Instant>,
    pub room_item_cursor: usize,
    pub inventory_cursor: usize,
}

impl GameState {
    pub fn new(manifest: Manifest) -> Self {
        Self {
            player_name: None,
            chat_history: Vec::new(),
            room_players: Vec::new(),
            group_id: None,
            group_leader: None,
            online_players_count: 1,
            hp: 100,
            max_hp: 100,
            manifest,
            room_npcs: Vec::new(),
            current_room_items: Vec::new(),
            inventory: Vec::new(),
            quests: Vec::new(),
            current_room_id: None,
            current_room_name: None,
            current_room_description: None,
            current_room_exits: HashMap::new(),
            focused_entity_id: None,
            action_logs: Vec::new(),
            active_dialogue: None,
            dialogue_closed_at: None,
            room_item_cursor: 0,
            inventory_cursor: 0,
        }
    }

    pub fn log_action(&mut self, text: String) {
        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
        self.action_logs.push(format!("[{}] {}", time_str, text));
    }

    pub fn is_npc_dialogue_available(&self) -> bool {
        if let Some(time) = self.dialogue_closed_at {
            if time.elapsed() < Duration::from_millis(300) {
                return false;
            }
        }

        true
    }
}
