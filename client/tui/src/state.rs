use api_client::client::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use tui_logger::TuiWidgetState;

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

pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected(String), // player name
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    Info,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationType,
    pub lifetime: u32,
}

pub struct NetworkState {
    pub client: Option<Arc<Mutex<Client>>>,
    pub connection_state: ConnectionState,
    pub server_ip: String,
    pub server_port: String,
    pub connected_at: Option<std::time::Instant>,
}

impl NetworkState {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            client: None,
            connection_state: ConnectionState::Disconnected,
            server_ip: ip,
            server_port: port,
            connected_at: None,
        }
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

pub struct UiState {
    pub game_scroll_offset: u16,
    pub show_debug: bool,
    pub show_help: bool,
    pub show_chat: bool,
    pub logger_state: TuiWidgetState,
    pub notifications: Vec<Notification>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            game_scroll_offset: 0,
            show_debug: false,
            show_help: false,
            show_chat: false,
            logger_state: TuiWidgetState::new(),
            notifications: Vec::new(),
        }
    }

    pub fn push_notification(
        &mut self,
        message: String,
        level: NotificationType,
        lifetime_ticks: u32,
    ) {
        self.notifications.push(Notification {
            message,
            level,
            lifetime: lifetime_ticks,
        });
    }
}

pub struct AppState {
    pub net: NetworkState,
    pub game: GameState,
    pub ui: UiState,
    pub assets: crate::assets::AssetManager,
    pub should_quit: bool,
}

impl AppState {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            net: NetworkState::new(ip, port),
            game: GameState::new(),
            ui: UiState::new(),
            assets: crate::assets::AssetManager::new(),
            should_quit: false,
        }
    }
}
