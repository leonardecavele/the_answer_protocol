use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::Lifecycle;
use crate::ui::components::lifecycle::EventFlow;
use crate::ui::components::scrollable::ScrollableComponent;
use crate::ui::theme::overlay_block;
use crate::ui::utils::wrap_slice_to_lines;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use mpsc::Sender;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Block;
use tokio::sync::mpsc;

const OVERLAY_WIDTH_PERCENTAGE: u16 = 80;
const OVERLAY_HEIGHT_PERCENTAGE: u16 = 80;

pub struct TraceOverlay;

impl Default for TraceOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceOverlay {
    pub fn new() -> Self {
        Self
    }
}

impl ScrollableComponent for TraceOverlay {
    fn get_area(&self, _state: &AppState, max_area: Rect) -> Rect {
        let overlay_width = max_area.width * OVERLAY_WIDTH_PERCENTAGE / 100;
        let overlay_height = max_area.height * OVERLAY_HEIGHT_PERCENTAGE / 100;

        Rect {
            x: max_area.x + (max_area.width - overlay_width) / 2,
            y: max_area.y + (max_area.height - overlay_height) / 2,
            width: overlay_width,
            height: overlay_height,
        }
    }

    fn get_block<'a>(&self, _state: &AppState) -> Block<'a> {
        overlay_block()
            .title(" Event history overlay (Press Ctrl+E or Esc to close) ")
            .style(Style::default().fg(Color::LightMagenta))
    }

    fn get_content<'a>(&self, state: &'a AppState, max_width: usize) -> Vec<Line<'a>> {
        let raw_lines = state
            .ui
            .trace_log
            .into_iter()
            .map(|line| format!("• {}", line))
            .collect::<Vec<_>>();

        wrap_slice_to_lines(&raw_lines, max_width)
    }
}

impl Lifecycle for TraceOverlay {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        _sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if let CrosstermEvent::Key(key) = event
            && key.code == KeyCode::Esc
        {
            state.ui.show_trace_log = false;
            return EventFlow::Consumed;
        }

        EventFlow::Ignored
    }
}
