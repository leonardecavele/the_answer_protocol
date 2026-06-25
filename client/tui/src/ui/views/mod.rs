pub mod game;
pub mod login;

use crate::events::ApplicationEvent;
use crate::states::AppState;
use crossterm::event::Event as CrosstermEvent;
pub use login::LoginView;
use ratatui::Frame;
use ratatui::layout::Rect;
use tokio::sync::mpsc;

/// The AppView trait defines how an interface screen should behave.
/// Each screen in the application (Login, Game, Settings, etc.) should implement this.
pub trait AppView {
    /// Renders the view using ratatui.
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect);

    /// Handles a terminal event specific to this view.
    /// It has mutable access to `AppState` to update the global state based on inputs.
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &mpsc::Sender<ApplicationEvent>,
    );

    /// Called on every system tick.
    fn on_tick(&mut self, _state: &mut AppState) {}
}
