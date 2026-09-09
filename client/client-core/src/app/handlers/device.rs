use crate::app::App;
use crate::renderer::components::Lifecycle;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use std::time::{Duration, Instant};

const KEY_RELEASE_DELAY: Duration = Duration::from_millis(120);

impl App {
    pub fn handle_device_event(&mut self, event: CrosstermEvent) {
        if let CrosstermEvent::Key(key_event) = event {
            let is_repeated = self.is_key_repeated(key_event.code);

            self.last_key = Some((key_event.code, Instant::now()));

            if is_repeated {
                return;
            }

            if key_event.modifiers == KeyModifiers::CONTROL && key_event.code == KeyCode::Char('c')
            {
                self.state.should_quit = true;
                return;
            }
            if key_event.modifiers == KeyModifiers::CONTROL && key_event.code == KeyCode::Char('e')
            {
                self.state.ui.show_trace_log = !self.state.ui.show_trace_log;
                return;
            }
        }

        let _ = self.view_manager.handle_device_event(
            &mut self.state,
            &event,
            &self.event_broker.sender(),
        );
    }

    fn is_key_repeated(&self, code: KeyCode) -> bool {
        if matches!(code, KeyCode::Char(_) | KeyCode::Backspace) {
            return false;
        }

        self.last_key
            .is_some_and(|(last_code, at)| last_code == code && at.elapsed() < KEY_RELEASE_DELAY)
    }
}
