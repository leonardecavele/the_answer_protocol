pub mod login;
pub mod game;

use crate::state::AppState;
use crate::events::AppEvent;
use ratatui::Frame;
use ratatui::layout::Rect;
use tokio::sync::mpsc;
use crossterm::event::Event;

#[async_trait::async_trait]
pub trait Component: Send + Sync {
    /// Handle a low-level crossterm terminal event (keyboard, mouse)
    async fn handle_event(&mut self, state: &mut AppState, event: &Event, tx: &mpsc::UnboundedSender<AppEvent>);
    
    /// Draw the component onto the screen
    fn draw(&mut self, state: &mut AppState, f: &mut Frame, area: Rect);
}
