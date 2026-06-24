use crate::states::app::AppState;
use crate::ui::components::Component;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};

pub struct RightPanelComponent;

impl RightPanelComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for RightPanelComponent {
    fn draw(&mut self, _state: &AppState, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Right Panel (Room Image) ");
        frame.render_widget(block, area);
    }
}
