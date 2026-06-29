use crate::states::app::AppState;
use crate::states::ui::GameFocus;
use crate::ui::components::Lifecycle;
use crate::ui::components::scrollable::ScrollableComponent;
use crate::ui::theme::default_block;
use crate::ui::utils::wrap_slice_to_lines;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Block;

pub struct ActionHistoryComponent;

impl ActionHistoryComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Lifecycle for ActionHistoryComponent {}

impl ScrollableComponent for ActionHistoryComponent {
    fn get_block<'a>(&self, state: &AppState) -> Block<'a> {
        let mut history_block = default_block()
            .title(" Action History ")
            .title_bottom(Line::from(" Press Ctrl + H to open help ").alignment(Alignment::Center));
        if state.ui.current_focus == GameFocus::ActionHistory {
            history_block = history_block.border_style(Style::default().fg(Color::Yellow));
        }
        history_block
    }

    fn get_content<'a>(&self, state: &'a AppState, max_width: usize) -> Vec<Line<'a>> {
        let raw_lines = state
            .game
            .action_logs
            .iter()
            .map(|line| format!("• {}", line))
            .collect::<Vec<_>>();

        wrap_slice_to_lines(&raw_lines, max_width)
    }
}
