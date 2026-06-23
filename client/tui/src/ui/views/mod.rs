pub mod login;
pub mod game;

pub use login::LoginView;
use ratatui::Frame;
use ratatui::layout::Rect;
use crossterm::event::Event as CrosstermEvent;
use tokio::sync::mpsc;
use crate::events::ApplicationEvent;
use crate::states::AppState;

/// The AppView trait defines how an interface screen should behave.
/// Each screen in the application (Login, Game, Settings, etc.) should implement this.
pub trait AppView {
    /// Renders the view using ratatui.
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect);

    /// Handles a terminal event specific to this view.
    /// It has mutable access to `AppState` to update the global state based on inputs.
    fn handle_terminal_event(&mut self, state: &mut AppState, event: &CrosstermEvent, event_sender: &mpsc::Sender<ApplicationEvent>);
}