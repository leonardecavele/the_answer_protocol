use crate::errors::ApplicationError;
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

pub const MAX_EVENT_HISTORY: usize = 100;

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
        let (manifest, err) = match crate::data::manifest::Manifest::load() {
            Ok(m) => (m, None),
            Err(e) => (crate::data::manifest::Manifest::default(), Some(e)),
        };

        let mut state = AppState::new(ip.clone(), port.clone(), manifest);
        if let Some(e) = err {
             state.ui.push(
                 crate::states::ui::Notification::error(e)
                     .with_duration(10000)
             );
        }

        Self {
            state,
            event_broker: EventBroker::new(),
            network_manager: None,
            active_view: Box::new(LoginView::new(ip, port)),
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
        if !matches!(event, ApplicationEvent::Tick) && !matches!(event, ApplicationEvent::Terminal(_)) {
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
                
                self.active_view.on_tick(&mut self.state);
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
                    self.state
                        .ui
                        .push(crate::states::ui::Notification::warning(error.to_string()));
                } else {
                    self.handle_api_response(envelope);
                }
            }
            ApplicationEvent::SendRawCommand(command) => {
                if let Some(request) = api_client::protocol::command::enums::ApiRequest::parse(&command) {
                    if let Some(network_manager) = &self.network_manager {
                        let envelope = crate::network::envelopes::RequestEnvelope::new(request);
                        network_manager.send_command(envelope);
                    }
                } else {
                    self.state.ui.push(crate::states::ui::Notification::warning(format!(
                        "Unknown or invalid command: {}",
                        command
                    )));
                }
            }
        }
    }
}
