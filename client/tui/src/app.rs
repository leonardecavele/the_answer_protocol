use api_client::client::Client;
use std::sync::Arc;
use tui_input::Input;
use tui_logger::TuiWidgetState;

pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected(String), // player name
}

pub enum Focus {
    Input,
    GameEvents,
    SystemLogs,
}

pub struct App {
    pub client: Option<Arc<tokio::sync::Mutex<Client>>>,
    pub state: ConnectionState,
    pub input: Input,
    pub messages: Vec<String>,
    pub scroll_offset: u16,
    pub should_quit: bool,
    pub server_ip: String,
    pub server_port: String,
    pub focus: Focus,
    pub logger_state: TuiWidgetState,
}

impl App {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            client: None,
            state: ConnectionState::Disconnected,
            input: Input::default(),
            messages: Vec::new(),
            scroll_offset: 0,
            should_quit: false,
            server_ip: ip,
            server_port: port,
            focus: Focus::Input,
            logger_state: TuiWidgetState::new(),
        }
    }

    pub fn push_message(&mut self, msg: String) {
        self.messages.push(msg);
        self.scroll_offset = 0; // auto-scroll to bottom on new message
        // keep only the last 1000 messages
        if self.messages.len() > 1000 {
            self.messages.remove(0);
        }
    }
}
