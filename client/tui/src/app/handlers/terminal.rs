use crate::app::runtime::App;
use crate::ui::components::Lifecycle;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};

impl App {
    pub(crate) fn handle_terminal_event(&mut self, event: CrosstermEvent) {
        if let CrosstermEvent::Key(key_event) = event {
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

        let _ = self.view_manager.handle_terminal_event(
            &mut self.state,
            &event,
            &self.event_broker.sender(),
        );
    }
}
