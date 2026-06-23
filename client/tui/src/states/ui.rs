use crate::events::types::NotificationType;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub struct Notification {
    pub id: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub expires_at: Instant,
}

impl Notification {
    pub fn new(id_opt: Option<String>, message: String, notif_type: NotificationType, duration_ms: u64) -> Self {
        Self {
            id: id_opt.unwrap_or_else(|| Uuid::new_v4().to_string()),
            message,
            notification_type: notif_type,
            expires_at: Instant::now() + Duration::from_millis(duration_ms),
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

    pub fn push_notification(
        &mut self,
        id_opt: Option<String>,
        notification_type: NotificationType,
        message: String,
        duration_ms: Option<u64>,
    ) {
        let duration = duration_ms.unwrap_or(5000);
        self.notifications.push(Notification::new(
            id_opt,
            message,
            notification_type,
            duration,
        ));
    }

    pub fn remove_notification(&mut self, target_id: &str) {
        self.notifications.retain(|n| n.id != target_id);
    }
}
