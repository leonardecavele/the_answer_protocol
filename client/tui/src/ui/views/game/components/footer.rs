use crate::states::app::AppState;
use crate::ui::components::Component;
use crate::ui::components::interactive::Interactive;
use crate::ui::components::text_input::TextInputComponent;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect};

pub struct FooterComponent {
    pub input: Interactive<TextInputComponent>,
}

impl FooterComponent {
    pub fn new() -> Self {
        let mut input = Interactive::new(TextInputComponent::new("Command"));
        input.inner.is_focused = true; // Focused by default
        Self { input }
    }
}

impl Component for FooterComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        // The text field takes up the entire area and uses its own border
        self.input.draw(state, frame, area);
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &tokio::sync::mpsc::Sender<crate::events::ApplicationEvent>,
    ) -> bool {
        // If mouse click on the footer, force focus
        if let CrosstermEvent::Mouse(mouse) = event {
            if self.input.is_mouse_over(mouse.column, mouse.row) {
                self.input.inner.is_focused = true;
            }
        }

        // Intercept the Enter key BEFORE passing it to TextInputComponent
        // (Because TextInput doesn't handle Enter, it returns false)
        if self.input.inner.is_focused {
            if let CrosstermEvent::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) = event
            {
                let command = self.input.inner.value.trim().to_string();
                if !command.is_empty() {
                    self.input.inner.value.clear();
                    let _ = event_sender.try_send(crate::events::ApplicationEvent::SendRawCommand(command));
                }
                return true;
            }
        }

        // Delegate to the interactive component (handles typing and backspace)
        self.input.handle_terminal_event(state, event, event_sender)
    }
}
