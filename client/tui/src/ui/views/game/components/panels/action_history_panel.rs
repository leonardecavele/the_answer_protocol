use crate::states::app::AppState;
use crate::states::game::GameFocus;
use crate::ui::components::{Lifecycle, ScrollableComponent};
use crate::ui::text::wrap_slice_to_lines;
use crate::ui::theme::{help_hint, panel_block};
use ratatui::text::Line;
use ratatui::widgets::Block;

#[derive(Default)]
pub struct ActionHistoryPanel;

impl ActionHistoryPanel {
    pub fn new() -> Self {
        Self
    }
}

impl Lifecycle for ActionHistoryPanel {}

impl ScrollableComponent for ActionHistoryPanel {
    fn is_scrollable(&self, state: &AppState) -> bool {
        state.game.focus() == GameFocus::ActionHistory
    }

    fn get_block<'a>(&self, state: &AppState) -> Block<'a> {
        panel_block(
            " Action History ",
            state.game.focus() == GameFocus::ActionHistory,
        )
        .title_bottom(help_hint())
    }

    fn get_content<'a>(&self, state: &'a AppState, max_width: usize) -> Vec<Line<'a>> {
        let raw_lines = state
            .game
            .action_log
            .into_iter()
            .map(|line| format!("• {}", line))
            .collect::<Vec<_>>();

        wrap_slice_to_lines(&raw_lines, max_width)
    }
}
