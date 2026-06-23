pub mod notifications;
pub mod event_overlay;
pub mod text_input;
pub mod button;

use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crossterm::event::Event as CrosstermEvent;
use ratatui::layout::Rect;
use ratatui::Frame;
use tokio::sync::mpsc;

/// The Component trait defines a reusable UI element.
pub trait Component {
    /// Renders the component on the given area.
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect);

    /// Indique si ce composant agit comme une "Modale" bloquante.
    /// S'il est bloquant, le gestionnaire principal arrêtera de propager l'événement.
    fn is_blocking(&self, _state: &AppState) -> bool {
        false
    }

    /// Handles an event, returning a boolean indicating if the event was consumed.
    /// If consumed, the event shouldn't be propagated further to other components or views.
    fn handle_event(&mut self, _state: &mut AppState, _event: &CrosstermEvent, _event_sender: &mpsc::Sender<ApplicationEvent>) -> bool {
        false
    }

    // --- MOUSE & CLICK SUPPORT ---
    
    /// Indicates if this component can receive focus or be clicked
    fn is_clickable(&self) -> bool {
        false
    }

    /// Returns the last area where this component was drawn
    fn get_last_area(&self) -> Option<Rect> {
        None
    }

    /// Saves the last area where this component was drawn (should be called in draw)
    fn set_last_area(&mut self, _area: Rect) {}

    /// Indicates whether the mouse position (col, row) is over the component
    fn is_mouse_over(&self, col: u16, row: u16) -> bool {
        if let Some(area) = self.get_last_area() {
            col >= area.x && col < area.x + area.width && row >= area.y && row < area.y + area.height
        } else {
            false
        }
    }
}
