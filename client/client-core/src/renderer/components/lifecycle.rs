use super::mouse::scroll_direction;
use crate::collections::Step;
use crate::events::ApplicationEvent;
use crate::states::AppState;
use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseButton, MouseEventKind};
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
        state: &mut AppState,
        event: &CrosstermEvent,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        match event {
            CrosstermEvent::Key(key) => self.on_key(state, key, sender),
            CrosstermEvent::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    self.on_click(state, mouse.column, mouse.row, sender)
                }
                kind => match scroll_direction(kind) {
                    Some(step) => self.on_scroll(state, step, mouse.column, mouse.row),
                    None => EventFlow::Ignored,
                },
            },
            _ => EventFlow::Ignored,
        }
    }

    fn on_key(
        &mut self,
        _state: &mut AppState,
        _key: &KeyEvent,
        _sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        EventFlow::Ignored
    }

    fn on_click(
        &mut self,
        _state: &mut AppState,
        _column: u16,
        _row: u16,
        _sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        EventFlow::Ignored
    }

    fn on_scroll(
        &mut self,
        _state: &mut AppState,
        _step: Step,
        _column: u16,
        _row: u16,
    ) -> EventFlow {
        EventFlow::Ignored
    }

    fn on_tick(&mut self, _state: &mut AppState, _sender: &Sender<ApplicationEvent>) {}
}
