pub mod notifications;
pub mod event_overlay;

use crate::states::app::AppState;
use crossterm::event::Event as CrosstermEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

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
    fn handle_event(&mut self, _state: &mut AppState, _event: &CrosstermEvent) -> bool {
        false
    }
}
