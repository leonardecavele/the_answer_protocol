use crate::collections::BoundedLog;
use crate::states::notification::Notifications;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Resize, StatefulImage};
use std::cell::RefCell;
use std::collections::HashMap;
//endregion

//region ImageManager
pub struct ImageManager {
    picker: Picker,
    cache: RefCell<HashMap<String, Option<(StatefulProtocol, u32, u32)>>>,
}

impl Default for ImageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageManager {
    pub fn new() -> Self {
        let picker = Picker::halfblocks();

        Self {
            picker,
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
    pub notifications: Notifications,
    pub image_manager: ImageManager,
    pub show_trace_log: bool,
    pub trace_log: BoundedLog<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiState {
    pub fn new() -> Self {
        Self {
            notifications: Notifications::default(),
            image_manager: ImageManager::new(),
            show_trace_log: false,
            trace_log: BoundedLog::with_max_capacity(100),
        }
    }
}
