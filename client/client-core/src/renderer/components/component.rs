use super::lifecycle::Lifecycle;
use crate::states::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;

pub trait Component: Lifecycle {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect);

    /// The area the component actually occupied at its last draw, which is not the area it was
    /// given: popups receive the whole screen and center themselves inside it.
    fn drawn_area(&self) -> Option<Rect> {
        None
    }
}
