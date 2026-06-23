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

    pub fn handle_event(&mut self, event: &crate::events::GameEvent) {
        use crate::events::GameEvent::*;
        match event {
            InventoryUpdate(items) => {
                self.inventory = items.clone();
            }
            UpdateOnlinePlayers(count) => {
                self.online_players = *count;
            }
            LocalChatSent(scope, msg) => {
                self.push_chat(scope.clone(), "You".to_string(), msg.clone());
            }
            UpdateGroup(group_name) => {
                self.group_name = group_name.clone();
            }
            UpdateRoomContext { room_id, room_display_name, npcs } => {
                self.current_room = room_id.clone();
                self.current_room_name = room_display_name.clone();
                self.npcs_in_room = npcs.clone();
            }
            UpdateStatus { hp, max_hp } => {
                self.hp = *hp;
                self.max_hp = *max_hp;
            }
            CommandResult(res) => {
                for line in res.lines() {
                    self.push_game_output(line.to_string());
                }
            }
            CommandError(err) => {
                let message = match err.code {
                    Some(code) => format!("[{}] Command error: {}", code, err.message),
                    None => format!("Command error: {}", err.message),
                };
                self.push_game_output(message);
            }
            UnknowCommand(cmd) => {
                self.push_game_output(format!("Unknown command: {}", cmd));
            }
            PushGameOutput(msg) => {
                self.push_game_output(msg.clone());
            }
        }
    }
}
