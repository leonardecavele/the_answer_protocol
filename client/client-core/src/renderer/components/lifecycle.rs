use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crossterm::event::Event as CrosstermEvent;
use mpsc::Sender;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum EventFlow {
    Consumed,
    Ignored,
}

impl EventFlow {
    pub fn is_consumed(self) -> bool {
        matches!(self, EventFlow::Consumed)
    }
}

pub trait Lifecycle {
    fn handle_device_event(
        &mut self,
        _state: &mut AppState,
        _event: &CrosstermEvent,
        _sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        EventFlow::Ignored
    }

    fn on_tick(&mut self, _state: &mut AppState, _sender: &Sender<ApplicationEvent>) {}
}
