use crate::events::ApplicationEvent;
use crate::renderer::components::{
    Component, EventFlow, Interactive, Lifecycle, TextInput, is_mouse_in_rect,
};
use crate::states::AppState;
use crate::states::game::GameFocus;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

pub enum FooterHit {
    CommandInput,
    None,
}

#[derive(Default)]
pub struct Footer {
    pub input: Interactive<TextInput>,
    area: Option<Rect>,
}

impl Footer {
    pub fn new() -> Self {
        let mut input = Interactive::new(TextInput::new("Command"));
        input.inner.is_focused = true;
        Self { input, area: None }
    }

    pub fn hit(&self, column: u16, row: u16) -> FooterHit {
        if let Some(area) = self.area
            && is_mouse_in_rect(column, row, area)
        {
            return FooterHit::CommandInput;
        }

        FooterHit::None
    }
}

impl Component for Footer {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.area = Some(area);
        self.input.inner.is_focused = state.game.focus() == GameFocus::Input;
        self.input.draw(state, frame, area);
    }
}

impl Lifecycle for Footer {
    fn handle_device_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &tokio::sync::mpsc::Sender<ApplicationEvent>,
    ) -> EventFlow {
        if state.game.focus() == GameFocus::Input
            && let CrosstermEvent::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) = event
        {
            let command = self.input.inner.value.trim().to_string();
            if !command.is_empty() {
                self.input.inner.value.clear();
                let _ = event_sender.try_send(ApplicationEvent::SendRawCommand(command));
            } else {
                state.game.set_focus(GameFocus::RightPanel);
            }
            return EventFlow::Consumed;
        }

        if state.game.focus() == GameFocus::Input {
            self.input.handle_device_event(state, event, event_sender)
        } else {
            EventFlow::Ignored
        }
    }
}
