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
        }
    }
}
