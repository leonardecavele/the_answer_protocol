pub mod scrollable;
pub mod widgets;

pub mod interactive;

use crate::states::app::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;

pub mod lifecycle;

pub use lifecycle::Lifecycle;

/// The Component trait defines a reusable UI element.
pub trait Component: Lifecycle {
    /// Renders the component on the given area.
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect);
}
