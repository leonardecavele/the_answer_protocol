use crate::states::app::AppState;
use crate::ui::components::Component;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};

pub struct CenterPanelComponent;

impl CenterPanelComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for CenterPanelComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Action History ");
        
        let logs: Vec<ratatui::text::Line> = state.game.action_logs
            .iter()
            .map(|log| ratatui::text::Line::from(log.as_str()))
            .collect();
        
        let logs_count = logs.len() as u16;
        let inner_height = area.height.saturating_sub(2);
        
        // Auto-scroll to bottom
        let scroll = if logs_count > inner_height {
            logs_count - inner_height
        } else {
            0
        };

        let list = ratatui::widgets::Paragraph::new(logs)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .scroll((scroll, 0));

        frame.render_widget(list, area);
    }
}
