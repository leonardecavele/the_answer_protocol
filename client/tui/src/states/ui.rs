use crate::events::types::NotificationType;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub const NOTIF_DEFAULT_DURATION_MS: u64 = 5000;

pub struct Notification {
    pub id: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub expires_at: Instant,
}

impl Notification {
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(
            None,
            message.into(),
            NotificationType::Information,
            NOTIF_DEFAULT_DURATION_MS,
        )
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(
            None,
            message.into(),
            NotificationType::Warning,
            NOTIF_DEFAULT_DURATION_MS,
        )
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(
            None,
            message.into(),
            NotificationType::Error,
            NOTIF_DEFAULT_DURATION_MS,
        )
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.expires_at = Instant::now() + Duration::from_millis(ms);
        self
    }

    pub fn new(
        id_opt: Option<String>,
        message: String,
        notif_type: NotificationType,
        duration_ms: u64,
    ) -> Self {
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

    pub fn push(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    pub fn remove_notification(&mut self, target_id: &str) {
        self.notifications.retain(|n| n.id != target_id);
    }
}
