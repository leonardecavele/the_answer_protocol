#[derive(Clone)]
pub enum ChatScope {
    Global,
    Room,
    Group,
    Private,
}

pub struct ChatEntry {
    pub scope: ChatScope,
    pub sender: String,
    pub message: String,
}

impl std::fmt::Display for ChatEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.scope {
            ChatScope::Global => "[Global]",
            ChatScope::Room => "[Room]",
            ChatScope::Group => "[Group]",
            ChatScope::Private => "[Private]",
        };
        write!(f, "{} {}: {}", prefix, self.sender, self.message)
    }
}

pub struct GameState {
    pub inventory: Vec<String>,
    pub online_players: u32,
    pub game_output: Vec<String>,
    pub chat_messages: Vec<ChatEntry>,
    pub hp: u32,
    pub max_hp: u32,
    pub group_name: Option<String>,
    pub current_room: String,
    pub current_room_name: String,
    pub npcs_in_room: Vec<String>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            inventory: Vec::new(),
            online_players: 0,
            game_output: Vec::new(),
            chat_messages: Vec::new(),
            hp: 100, // Default mock values
            max_hp: 100,
            group_name: None,
            current_room: "".to_string(),
            current_room_name: "".to_string(),
            npcs_in_room: vec![],
        }
    }

    pub fn push_game_output(&mut self, msg: String) {
        self.game_output.push(msg);
        if self.game_output.len() > 1000 {
            self.game_output.remove(0);
        }
    }

    pub fn push_chat(&mut self, scope: ChatScope, sender: String, message: String) {
        let entry = ChatEntry {
            scope,
            sender,
            message,
        };
        self.push_game_output(entry.to_string());
        
        self.chat_messages.push(entry);
        if self.chat_messages.len() > 500 {
            self.chat_messages.remove(0);
        }
    }
}
