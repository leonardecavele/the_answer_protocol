use api_client::client::Client;
use std::sync::Arc;
use tui_input::Input;
use tui_logger::TuiWidgetState;

#[derive(PartialEq)]
pub enum Screen {
    Login,
    Game,
}

#[derive(PartialEq, Clone)]
pub enum LoginField {
    Username,
    Address,
    Port,
    Button,
}

#[derive(PartialEq)]
pub enum GameFocus {
    Input,
    Actions,
}

pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected(String), // player name
}

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

pub const ACTIONS: &[&str] = &["WHO", "LOOK", "ATTACK", "QUEST", "QUIT"];

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

pub struct App {
    pub client: Option<Arc<tokio::sync::Mutex<Client>>>,
    pub state: ConnectionState,
    pub screen: Screen,

    // Login screen
    pub login_field: LoginField,
    pub username_input: Input,
    pub address_input: Input,
    pub port_input: Input,

    // Game screen
    pub game_focus: GameFocus,
    pub input: Input,
    pub selected_action: usize,
    pub game_output: Vec<String>,
    pub chat_messages: Vec<ChatEntry>,
    pub inventory: Vec<String>,

    // Shared
    pub online_players: u32,
    pub game_scroll_offset: u16,
    pub chat_scroll_offset: u16,
    pub should_quit: bool,
    pub server_ip: String,
    pub server_port: String,
    pub show_debug: bool,
    pub show_help: bool,
    pub show_chat: bool,
    pub logger_state: TuiWidgetState,
    pub notifications: Vec<Notification>,
}

impl App {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            client: None,
            state: ConnectionState::Disconnected,
            screen: Screen::Login,
            login_field: LoginField::Username,
            username_input: Input::default(),
            address_input: Input::from(ip.clone()),
            port_input: Input::from(port.clone()),
            game_focus: GameFocus::Input,
            input: Input::default(),
            selected_action: 0,
            game_output: Vec::new(),
            chat_messages: Vec::new(),
            inventory: Vec::new(),
            online_players: 0,
            game_scroll_offset: 0,
            chat_scroll_offset: 0,
            should_quit: false,
            server_ip: ip,
            server_port: port,
            show_debug: false,
            show_help: false,
            show_chat: false,
            logger_state: TuiWidgetState::new(),
            notifications: Vec::new(),
        }
    }

    pub fn push_game_output(&mut self, msg: String) {
        self.game_output.push(msg);
        self.game_scroll_offset = 0;
        if self.game_output.len() > 1000 {
            self.game_output.remove(0);
        }
    }

    pub fn push_chat(&mut self, scope: ChatScope, sender: String, message: String) {
        self.chat_messages.push(ChatEntry {
            scope,
            sender,
            message,
        });
        self.chat_scroll_offset = 0;
        if self.chat_messages.len() > 500 {
            self.chat_messages.remove(0);
        }
    }

    pub fn push_notification(&mut self, message: String, level: NotificationType, lifetime_ticks: u32) {
        self.notifications.push(Notification {
            message,
            level,
            lifetime: lifetime_ticks,
        });
    }
}
