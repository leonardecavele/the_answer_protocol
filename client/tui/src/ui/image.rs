use crate::ui::layout::fit_area;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Resize, StatefulImage};
use std::cell::RefCell;
use std::collections::HashMap;

const CELL_HEIGHT_OVER_WIDTH: f32 = 2.0;

struct CachedImage {
    protocol: StatefulProtocol,
    width: u32,
    height: u32,
}

type ImageCache = HashMap<String, Option<CachedImage>>;

pub struct ImageRenderer {
    picker: Picker,
    cache: RefCell<ImageCache>,
}

impl Default for ImageRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageRenderer {
    pub fn new() -> Self {
        Self {
            picker: Picker::halfblocks(),
            cache: RefCell::new(ImageCache::new()),
        }
    }

    pub fn aspect_ratio(&self, path: &str) -> Option<f32> {
        self.ensure_loaded(path);

        let cache = self.cache.borrow();
        let cached = cache.get(path)?.as_ref()?;

        if cached.height == 0 {
            return None;
        }

        Some(cached.width as f32 / (cached.height as f32 / CELL_HEIGHT_OVER_WIDTH))
    }

    pub fn draw_fitted(&self, frame: &mut Frame, area: Rect, path: &str, resize: Resize) -> Rect {
        let image_area = match self.aspect_ratio(path) {
            Some(aspect) => fit_area(area, aspect),
            None => area,
        };

        let mut cache = self.cache.borrow_mut();
        if let Some(Some(cached)) = cache.get_mut(path) {
            let widget = StatefulImage::default().resize(resize);
            frame.render_stateful_widget(widget, image_area, &mut cached.protocol);
        }

        image_area
    }

    fn ensure_loaded(&self, path: &str) {
        let mut cache = self.cache.borrow_mut();
        if cache.contains_key(path) {
            return;
        }

        let cached = match image::open(path) {
            Ok(image) => {
                let width = image.width();
                let height = image.height();

                Some(CachedImage {
                    protocol: self.picker.new_resize_protocol(image),
                    width,
                    height,
                })
            }
            Err(_) => None,
        };

        cache.insert(path.to_string(), cached);
    }
}
