use tui_logger::TuiWidgetState;

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

#[derive(Debug, Clone, PartialEq)]
pub enum GameFocus {
    Input,
    Scene,
    SystemLogs,
}

pub struct UiState {
    pub game_scroll_offset: u16,
    pub show_debug: bool,
    pub show_help: bool,
    pub show_chat: bool,
    pub logger_state: TuiWidgetState,
    pub notifications: Vec<Notification>,
    
    // Extracted from GameComponent
    pub game_focus: GameFocus,
    pub input: tui_input::Input,
    pub selected_entity_idx: Option<usize>,
    pub context_menu_open: bool,
    pub context_menu_idx: usize,
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
            
            game_focus: GameFocus::Input,
            input: tui_input::Input::default(),
            selected_entity_idx: None,
            context_menu_open: false,
            context_menu_idx: 0,
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
