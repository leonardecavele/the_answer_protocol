use ratatui::text::Line;
use crate::states::app::AppState;
use ratatui::layout::Rect;
use ratatui::Frame;
use ratatui_image::{Resize, StatefulImage};

/// Wrap text efficiently to match the exact visual lines it will take on screen.
pub fn wrap_str_to_lines<'a, 'b>(text: &'a str, max_width: usize) -> Vec<Line<'b>> {
    let wrapped = textwrap::wrap(text, max_width);
    wrapped
        .into_iter()
        .map(|w| Line::from(w.into_owned()))
        .collect()
}

/// Helper to wrap a slice of strings into a continuous vector of lines.
pub fn wrap_slice_to_lines(strs: &[String], max_width: usize) -> Vec<Line<'static>> {
    let mut visual_lines = Vec::new();
    for text in strs {
        let wrapped = textwrap::wrap(text, max_width);
        for w in wrapped {
            visual_lines.push(Line::from(w.into_owned()));
        }
    }
    visual_lines
}

/// Renders an image using ratatui-image, handling caching automatically
pub fn render_image(
    state: &AppState,
    frame: &mut Frame,
    area: Rect,
    path: &str,
    resize: Resize,
) {
    state.ui.ensure_image_loaded(path);
    let mut cache = state.ui.image_cache.borrow_mut();
    if let Some(Some((protocol, _, _))) = cache.get_mut(path) {
        let image_widget = StatefulImage::default().resize(resize);
        frame.render_stateful_widget(image_widget, area, protocol);
    }
}
