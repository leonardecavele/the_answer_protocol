use super::lifecycle::Lifecycle;
use super::mouse::is_mouse_in_rect;
use crate::states::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;

pub trait Component: Lifecycle {
    /// The area the component actually occupied at its last draw, which is not the area it was
    /// given: popups receive the whole screen and center themselves inside it.
    fn drawn_area(&self) -> Option<Rect> {
        None
    }

    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect);

    fn hit(&self, column: u16, row: u16) -> bool {
        self.drawn_area()
            .is_some_and(|area| is_mouse_in_rect(column, row, area))
    }
}
