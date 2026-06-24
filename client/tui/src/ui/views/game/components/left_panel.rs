use crate::states::app::AppState;
use crate::ui::components::Component;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};

pub struct LeftPanelComponent;

impl LeftPanelComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for LeftPanelComponent {
    fn draw(&mut self, _state: &AppState, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title(" Left Panel ");
        frame.render_widget(block, area);
    }
}
