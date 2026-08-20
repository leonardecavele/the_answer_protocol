use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::GameFocus;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::components::interactive::Interactive;
use crate::ui::components::lifecycle::EventFlow;
use crate::ui::components::widgets::text_input::TextInput;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

pub struct Footer {
    pub input: Interactive<TextInput>,
}

impl Default for Footer {
    fn default() -> Self {
        Self::new()
    }
}

impl Footer {
    pub fn new() -> Self {
        let mut input = Interactive::new(TextInput::new("Command"));
        input.inner.is_focused = true;
        Self { input }
    }
}

impl Component for Footer {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.input.inner.is_focused = state.game.focus == GameFocus::Input;
        self.input.draw(state, frame, area);
    }
}

impl Lifecycle for Footer {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &tokio::sync::mpsc::Sender<ApplicationEvent>,
    ) -> EventFlow {
        if state.game.focus == GameFocus::Input
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
                state.game.focus = GameFocus::RightPanel;
            }
            return EventFlow::Consumed;
        }

        if state.game.focus == GameFocus::Input {
            self.input.handle_terminal_event(state, event, event_sender)
        } else {
            EventFlow::Ignored
        }
    }
}
