use crate::app::App;
use crate::ui::components::Component;
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
                self.state.ui.show_event_overlay = !self.state.ui.show_event_overlay;
                return;
            }
        }

        if self.event_overlay.is_blocking(&self.state) {
            let _ = self.event_overlay.handle_terminal_event(
                &mut self.state,
                &event,
                &self.event_broker.sender(),
            );
            return;
        }

        if let CrosstermEvent::Mouse(mouse_event) = event {
            if self
                .notification_overlay
                .is_mouse_over(mouse_event.column, mouse_event.row)
            {
                if self.notification_overlay.handle_terminal_event(
                    &mut self.state,
                    &event,
                    &self.event_broker.sender(),
                ) {
                    return;
                }
            }
        }

        self.active_view
            .handle_terminal_event(&mut self.state, &event, &self.event_broker.sender());
    }
}
