pub mod button;
pub mod event_overlay;
pub mod notifications;
pub mod text_input;

pub mod interactive;

use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crossterm::event::Event as CrosstermEvent;
use ratatui::Frame;
use ratatui::layout::Rect;
use tokio::sync::mpsc;

/// Helper function to check if the mouse coordinates fall within a Rect
pub fn is_mouse_in_rect(col: u16, row: u16, area: Rect) -> bool {
    col >= area.x && col < area.x + area.width && row >= area.y && row < area.y + area.height
}

/// The Component trait defines a reusable UI element.
pub trait Component {
    /// Renders the component on the given area.
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect);

    /// Indicates whether this component acts as a blocking "Modal".
    /// If it is blocking, the main event handler will stop propagating the event.
    fn is_blocking(&self, _state: &AppState) -> bool {
        false
    }

    /// Handles an event, returning a boolean indicating if the event was consumed.
    /// If consumed, the event shouldn't be propagated further to other components or views.
    fn handle_terminal_event(
        &mut self,
        _state: &mut AppState,
        _event: &CrosstermEvent,
        _event_sender: &mpsc::Sender<ApplicationEvent>,
    ) -> bool {
        false
    }
}
