use crate::states::app::AppState;
use crate::ui::components::Component;
use crossterm::event::Event as CrosstermEvent;
// No need to import keyboard events anymore as we do not manage them manually
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem};
use ratatui::Frame;

pub struct EventOverlayComponent;

impl EventOverlayComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for EventOverlayComponent {
    fn handle_terminal_event(&mut self, _state: &mut AppState, _event: &CrosstermEvent, _event_sender: &tokio::sync::mpsc::Sender<crate::events::ApplicationEvent>) -> bool {
        false
    }

    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        if !state.ui.show_event_overlay {
            return;
        }

        // Create an area in the center of the screen
        let overlay_width = area.width * 80 / 100;
        let overlay_height = area.height * 80 / 100;
        let overlay_x = (area.width - overlay_width) / 2;
        let overlay_y = (area.height - overlay_height) / 2;

        let overlay_area = Rect {
            x: overlay_x,
            y: overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        let block = Block::default()
            .title(" Event History Overlay (Press Ctrl+E to close) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightMagenta));

        let items: Vec<ListItem> = state
            .ui
            .event_history
            .iter()
            .map(|evt| ListItem::new(evt.clone()))
            .collect();

        let list = List::new(items).block(block);

        // Clear the area and draw the list
        frame.render_widget(Clear, overlay_area);
        frame.render_widget(list, overlay_area);
    }

    fn is_blocking(&self, state: &AppState) -> bool {
        state.ui.show_event_overlay
    }
}
