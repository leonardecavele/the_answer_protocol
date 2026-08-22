use crate::states::app::AppState;
use crate::ui::components::Lifecycle;
use ratatui::Frame;
use ratatui::layout::Rect;

pub trait Component: Lifecycle {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect);
}
