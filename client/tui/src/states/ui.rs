use crate::events::types::NotificationType;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub const NOTIF_DEFAULT_DURATION_MS: u64 = 5000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameFocus {
    Input,
    RightPanel,
    NpcList,
    RoomItemsList,
    InventoryGrid,
    ActionHistory,
}

impl Default for GameFocus {
    fn default() -> Self {
        GameFocus::Input
    }
}

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
    pub show_event_overlay: bool,
    pub event_history: Vec<String>,
    pub image_picker: Picker,
    pub image_cache: RefCell<HashMap<String, Option<(StatefulProtocol, u32, u32)>>>,
    pub current_focus: GameFocus,
    pub active_npc_popup: Option<String>,
    pub active_item_popup: Option<String>,
    pub active_item_view_popup: Option<String>,
    pub show_help_overlay: bool,
}

impl UiState {
    pub fn new() -> Self {
        let picker = Picker::halfblocks();

        Self {
            notifications: Vec::new(),
            show_event_overlay: false,
            event_history: Vec::new(),
            image_picker: picker,
            image_cache: RefCell::new(HashMap::new()),
            current_focus: GameFocus::default(),
            active_npc_popup: None,
            active_item_popup: None,
            active_item_view_popup: None,
            show_help_overlay: false,
        }
    }

    pub fn push(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    pub fn remove_notification(&mut self, target_id: &str) {
        self.notifications.retain(|n| n.id != target_id);
    }

    pub fn ensure_image_loaded(&self, path: &str) {
        let mut cache = self.image_cache.borrow_mut();
        if !cache.contains_key(path) {
            match image::open(path) {
                Ok(dyn_img) => {
                    let w = dyn_img.width();
                    let h = dyn_img.height();
                    let protocol = self.image_picker.new_resize_protocol(dyn_img);
                    cache.insert(path.to_string(), Some((protocol, w, h)));
                }
                Err(_) => {
                    cache.insert(path.to_string(), None);
                }
            }
        }
    }

    pub fn get_image_dimensions(&self, path: &str) -> Option<(u32, u32)> {
        self.ensure_image_loaded(path);
        let cache = self.image_cache.borrow();
        if let Some(Some((_, w, h))) = cache.get(path) {
            Some((*w, *h))
        } else {
            None
        }
    }
}
