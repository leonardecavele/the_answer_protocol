use crate::events::types::NotificationType;
use ratatui::layout::Rect;
use ratatui::Frame;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Resize, StatefulImage};
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub const NOTIF_DEFAULT_DURATION_MS: u64 = 5000;

//region Notifications
// TODO: Changer l'emplacement des notifications
pub struct Notification {
    pub id: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub expires_at: Instant,
    pub duration: u64,
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

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(
            None,
            message.into(),
            NotificationType::Success,
            NOTIF_DEFAULT_DURATION_MS,
        )
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(None, message.into(), NotificationType::Error, 0)
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
            duration: duration_ms,
        }
    }
}

pub struct Notifications {
    notifications: Vec<Notification>,
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.notifications.is_empty()
    }

    pub fn take(&self, n: usize) -> Vec<&Notification> {
        self.notifications.iter().rev().take(n).collect::<Vec<_>>()
    }

    pub fn push(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    pub fn remove(&mut self, target_id: &str) {
        self.notifications.retain(|n| n.id != target_id);
    }

    pub fn remove_expired(&mut self) {
        self.notifications
            .retain(|n| Instant::now() < n.expires_at || n.duration == 0);
    }
}
//endregion

//region ImageManager
pub struct ImageManager {
    picker: Picker,
    cache: RefCell<HashMap<String, Option<(StatefulProtocol, u32, u32)>>>,
}

impl ImageManager {
    pub fn new() -> Self {
        let picker = Picker::halfblocks();

        Self {
            picker: picker,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn ensure_loaded(&self, path: &str) {
        let mut cache = self.cache.borrow_mut();
        if !cache.contains_key(path) {
            match image::open(path) {
                Ok(dyn_img) => {
                    let w = dyn_img.width();
                    let h = dyn_img.height();
                    let protocol = self.picker.new_resize_protocol(dyn_img);
                    cache.insert(path.to_string(), Some((protocol, w, h)));
                }
                Err(_) => {
                    cache.insert(path.to_string(), None);
                }
            }
        }
    }

    pub fn get_dimensions(&self, path: &str) -> Option<(u32, u32)> {
        self.ensure_loaded(path);
        let cache = self.cache.borrow();
        if let Some(Some((_, w, h))) = cache.get(path) {
            Some((*w, *h))
        } else {
            None
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, path: &str, resize: Resize) {
        self.ensure_loaded(path);
        let mut cache = self.cache.borrow_mut();
        if let Some(Some((protocol, _, _))) = cache.get_mut(path) {
            let image_widget = StatefulImage::default().resize(resize);
            frame.render_stateful_widget(image_widget, area, protocol);
        }
    }
}
//endregion

pub struct UiState {
    pub notification: Notifications,
    pub image_manager: ImageManager,
    pub show_event_overlay: bool,
    pub event_history: Vec<String>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            notification: Notifications::new(),
            image_manager: ImageManager::new(),
            show_event_overlay: false,
            event_history: Vec::new(),
        }
    }
}
