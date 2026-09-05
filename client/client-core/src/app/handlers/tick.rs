use crate::app::App;
use crate::renderer::components::Lifecycle;

impl App {
    pub fn handle_tick(&mut self) {
        self.state.ui.notifications.retain_active();

        let sender = self.event_broker.sender();
        self.view_manager.on_tick(&mut self.state, &sender);
    }
}
