use crate::events::ApplicationEvent;
use crate::renderer::components::{
    Component, EventFlow, Lifecycle, NotificationsOverlay, Scrollable, TraceOverlay,
};
use crate::renderer::layout::{MIN_COLUMNS, MIN_ROWS, interface_area};
use crate::renderer::theme::too_small_hint;
use crate::renderer::views::login::LoginView;
use crate::states::app::AppState;
use crossterm::event::Event as CrosstermEvent;
use ratatui::Frame;
use ratatui::layout::Rect;
use tokio::sync::mpsc::Sender;

pub struct ViewManager {
    active_view: Box<dyn Component>,
    event_overlay: Scrollable<TraceOverlay>,
    notification_overlay: NotificationsOverlay,
}

impl ViewManager {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            active_view: Box::new(LoginView::new(ip, port)),
            event_overlay: Scrollable::new(TraceOverlay::new()),
            notification_overlay: NotificationsOverlay::new(),
        }
    }

    pub fn set_view(&mut self, view: Box<dyn Component>) {
        self.active_view = view;
    }
}

impl Component for ViewManager {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let Some(area) = interface_area(area) else {
            frame.render_widget(too_small_hint(MIN_COLUMNS, MIN_ROWS), area);
            return;
        };

        self.active_view.draw(state, frame, area);

        if state.ui.show_trace_log {
            self.event_overlay.draw(state, frame, area);
        }

        self.notification_overlay.draw(state, frame, area);
    }
}

impl Lifecycle for ViewManager {
    fn handle_device_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if state.ui.show_trace_log {
            let _ = self.event_overlay.handle_device_event(state, event, sender);

            return EventFlow::Consumed;
        }

        if self
            .notification_overlay
            .handle_device_event(state, event, sender)
            .is_consumed()
        {
            return EventFlow::Consumed;
        }

        self.active_view.handle_device_event(state, event, sender)
    }

    fn on_tick(&mut self, state: &mut AppState, sender: &Sender<ApplicationEvent>) {
        self.active_view.on_tick(state, sender)
    }
}
