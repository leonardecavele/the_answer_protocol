use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crossterm::event::Event as CrosstermEvent;
use mpsc::Sender;
use tokio::sync::mpsc;

pub trait Lifecycle {
    fn handle_terminal_event(
        &mut self,
        _state: &mut AppState,
        _event: &CrosstermEvent,
        _sender: &Sender<ApplicationEvent>,
    ) -> bool {
        false
    }

    fn on_tick(&mut self, _state: &mut AppState) {}
}
