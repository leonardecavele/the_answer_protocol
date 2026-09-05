use crate::ui::layout::{centered_rect, fit_area};
use client_core::Assets;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Resize, StatefulImage};
use std::cell::RefCell;
use std::collections::HashMap;
use tracing::warn;

const CELL_HEIGHT_OVER_WIDTH: f32 = 2.0;
const MAX_IMAGE_CELLS: u32 = u16::MAX as u32;

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

    pub fn aspect_ratio(&self, assets: &Assets, path: &str) -> Option<f32> {
        self.ensure_loaded(assets, path);

        let cache = self.cache.borrow();
        let cached = cache.get(path)?.as_ref()?;

        if cached.height == 0 {
            return None;
        }

        Some(cached.width as f32 / (cached.height as f32 / CELL_HEIGHT_OVER_WIDTH))
    }

    fn bounded_area(area: Rect) -> Rect {
        let cells = u32::from(area.width) * u32::from(area.height);

        if cells <= MAX_IMAGE_CELLS {
            return area;
        }

        let scale = (MAX_IMAGE_CELLS as f32 / cells as f32).sqrt();

        centered_rect(
            area,
            (area.width as f32 * scale) as u16,
            (area.height as f32 * scale) as u16,
        )
    }

    pub fn draw_fitted(
        &self,
        frame: &mut Frame,
        area: Rect,
        assets: &Assets,
        path: &str,
        resize: Resize,
    ) -> Rect {
        let area = Self::bounded_area(area);

        let image_area = match self.aspect_ratio(assets, path) {
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

    fn ensure_loaded(&self, assets: &Assets, path: &str) {
        if self.cache.borrow().contains_key(path) {
            return;
        }

        let cached = self.decode(assets, path);
        self.cache.borrow_mut().insert(path.to_string(), cached);
    }

    fn decode(&self, assets: &Assets, path: &str) -> Option<CachedImage> {
        let Some(bytes) = assets.read(path) else {
            warn!("Missing asset: {}", path);
            return None;
        };

        let image = match image::load_from_memory(&bytes) {
            Ok(image) => image,
            Err(error) => {
                warn!("Cannot decode asset {}: {}", path, error);
                return None;
            }
        };

        let width = image.width();
        let height = image.height();

        Some(CachedImage {
            protocol: self.picker.new_resize_protocol(image),
            width,
            height,
        })
    }
}
