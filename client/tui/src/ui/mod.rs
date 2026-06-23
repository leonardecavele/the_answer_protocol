pub mod components;
pub mod views;

use crate::states::app::AppState;
use crossterm::event::Event as CrosstermEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

/// The AppView trait defines how an interface screen should behave.
/// Each screen in the application (Login, Game, Settings, etc.) should implement this.
pub trait AppView {
    /// Renders the view using ratatui.
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect);

    /// Handles a terminal event specific to this view.
    /// It has mutable access to `AppState` to update the global state based on inputs.
    fn handle_event(&mut self, state: &mut AppState, event: &CrosstermEvent);
}
