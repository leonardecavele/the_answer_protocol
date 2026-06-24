use std::time::Instant;

pub const END_OF_DIALOGUE_TAG: &str = "[end of dialogue]";

#[derive(Debug, Clone, PartialEq)]
pub enum DialogueClearMode {
    AlwaysClear,
    ClearOnEndTag,
}

impl Default for DialogueClearMode {
    fn default() -> Self {
        DialogueClearMode::ClearOnEndTag
    }
}

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
    pub group_members: Vec<String>,
    pub online_players_count: u32,
    pub manifest: crate::data::manifest::Manifest,
    pub room_npcs: Vec<String>,
    pub current_room_id: Option<String>,
    pub current_room_name: Option<String>,
    pub current_room_description: Option<String>,
    pub current_room_exits: std::collections::HashMap<String, String>,
    pub focused_entity_id: Option<String>,
    pub action_logs: Vec<String>,
    pub active_dialogue: Option<DialogueState>,
    pub dialogue_clear_mode: DialogueClearMode,
}

impl GameState {
    pub fn new(manifest: crate::data::manifest::Manifest) -> Self {
        Self {
            player_name: None,
            chat_history: Vec::new(),
            room_players: Vec::new(),
            group_members: Vec::new(),
            online_players_count: 0,
            manifest,
            room_npcs: Vec::new(),
            current_room_id: None,
            current_room_name: None,
            current_room_description: None,
            current_room_exits: std::collections::HashMap::new(),
            focused_entity_id: None,
            action_logs: Vec::new(),
            active_dialogue: None,
            dialogue_clear_mode: DialogueClearMode::default(),
        }
    }

    pub fn log_action(&mut self, text: String) {
        self.action_logs.push(text);
    }
}
