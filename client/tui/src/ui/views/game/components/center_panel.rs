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
            .title(" Action History ")
            .title_bottom(ratatui::text::Line::from(" Press Ctrl + H to open help ").alignment(ratatui::layout::Alignment::Center));

        let inner_area = block.inner(area);
        let max_width = inner_area.width as usize;

        let visual_lines =
            crate::ui::utils::wrap_slice_to_lines(&state.game.action_logs, max_width);

        let logs_count = visual_lines.len() as u16;
        let inner_height = inner_area.height;

        // Auto-scroll to bottom
        let scroll = if logs_count > inner_height {
            logs_count - inner_height
        } else {
            0
        };

        let list = ratatui::widgets::Paragraph::new(visual_lines)
            .block(block)
            .scroll((scroll, 0));

        frame.render_widget(list, area);
    }
}
