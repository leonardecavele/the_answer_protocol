pub mod scrollable;
pub mod widgets;

pub mod interactive;

use crate::states::app::AppState;
use ratatui::layout::Rect;
use ratatui::Frame;

pub mod lifecycle;

pub use lifecycle::Lifecycle;

pub trait Component: Lifecycle {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect);
}
