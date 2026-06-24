use crate::constants::{MAX_EVENT_HISTORY, TICK_RATE};
use crate::errors::ApplicationError;
use crate::events::NotificationType;
use crate::events::{ApplicationEvent, EventBroker, SystemEvent};
use crate::network::NetworkManager;
use crate::states::app::AppState;
use crate::ui::components::Component;
use crate::ui::components::event_overlay::EventOverlayComponent;
use crate::ui::components::notifications::NotificationComponent;
use crate::ui::views::AppView;
use crate::ui::views::LoginView;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::Instant;

pub mod handlers;

pub struct App {
    pub state: AppState,
    pub event_broker: EventBroker,
    // Store the network manager to keep its background task alive
    pub network_manager: Option<NetworkManager>,
    pub active_view: Box<dyn AppView>,
    pub event_overlay: EventOverlayComponent,
    pub notification_overlay: NotificationComponent,
}

impl App {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            state: AppState::new(ip, port),
            event_broker: EventBroker::new(TICK_RATE),
            network_manager: None,
            active_view: Box::new(LoginView::new()),
            event_overlay: EventOverlayComponent::new(),
            notification_overlay: NotificationComponent::new(),
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), ApplicationError> {
        while !self.state.should_quit {
            terminal.draw(|frame| {
                let area = frame.area();
                self.active_view.draw(&self.state, frame, area);
                self.event_overlay.draw(&self.state, frame, area);
                self.notification_overlay.draw(&self.state, frame, area);
            })?;

            let application_event = self.event_broker.next_event().await?;

            self.update(application_event);
        }

        Ok(())
    }

    fn update(&mut self, event: ApplicationEvent) {
        if !matches!(event, ApplicationEvent::Tick) {
            self.state
                .ui
                .event_history
                .insert(0, format!("{:?}", event));
            if self.state.ui.event_history.len() > MAX_EVENT_HISTORY {
                self.state.ui.event_history.truncate(MAX_EVENT_HISTORY);
            }
        }

        match event {
            ApplicationEvent::Tick => {
                self.state
                    .ui
                    .notifications
                    .retain(|n| Instant::now() < n.expires_at);
            }
            ApplicationEvent::Terminal(crossterm_event) => {
                self.handle_terminal_event(crossterm_event);
            }
            ApplicationEvent::System(system_event) => match system_event {
                SystemEvent::QuitRequested => {
                    self.state.should_quit = true;
                }
            },
            ApplicationEvent::Network(network_event) => {
                self.handle_network_event(network_event);
            }
            ApplicationEvent::ApiResponse(envelope) => {
                if let Some(error) = envelope.response.get_error() {
                    self.state.ui.push_notification(
                        None,
                        NotificationType::Warning,
                        error.message.clone(),
                        None,
                    );
                } else {
                    self.handle_api_response(envelope);
                }
            }
        }
    }
}
