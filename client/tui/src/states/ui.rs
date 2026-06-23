use crate::events::types::NotificationType;
use uuid::Uuid;

pub struct Notification {
    pub id: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub remaining_ticks: u32,
}

impl Notification {
    pub fn new(id_opt: Option<String>, message: String, notif_type: NotificationType, ticks: u32) -> Self {
        Self {
            id: id_opt.unwrap_or_else(|| Uuid::new_v4().to_string()),
            message,
            notification_type: notif_type,
            remaining_ticks: ticks,
        }
    }
}

pub struct UiState {
    pub notifications: Vec<Notification>,
    pub event_history: Vec<String>,
    pub show_event_overlay: bool,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            event_history: Vec::new(),
            show_event_overlay: false,
        }
    }
}
