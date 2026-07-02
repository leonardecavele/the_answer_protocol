use crate::data::manifest::Manifest;
use crate::errors::ApplicationError;
use crate::events::{ApplicationEvent, EventBroker};
use crate::network::NetworkManager;
use crate::network::envelopes::RequestEnvelope;
use crate::states::app::AppState;
use crate::states::ui::Notification;
use crate::ui::components::Component;
use crate::ui::components::scrollable::Scrollable;
use crate::ui::components::widgets::event_overlay::EventOverlayComponent;
use crate::ui::components::widgets::notifications::NotificationComponent;
use crate::ui::views::login::LoginView;
use api_client::protocol::command::enums::ApiRequest;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::sync::Arc;
use std::time::Instant;

pub mod handlers;

pub const MAX_EVENT_HISTORY: usize = 100;

pub struct App {
    pub state: AppState,
    pub event_broker: EventBroker,
    pub network_manager: Option<NetworkManager>,
    pub active_view: Box<dyn Component>,
    pub event_overlay: Scrollable<EventOverlayComponent>,
    pub notification_overlay: NotificationComponent,
}

impl App {
    pub fn new(ip: String, port: String) -> Self {
        let (manifest, err) = match Manifest::load() {
            Ok(manifest) => (manifest, None),
            Err(error) => (Manifest::default(), Some(error)),
        };

        let mut state = AppState::new(ip.clone(), port.clone(), Arc::new(manifest));
        if let Some(error) = err {
            state
                .ui
                .push(Notification::error(error).with_duration(10000));
        }

        Self {
            state,
            event_broker: EventBroker::new(),
            network_manager: None,
            active_view: Box::new(LoginView::new(ip, port)),
            event_overlay: Scrollable::new(EventOverlayComponent::new()),
            notification_overlay: NotificationComponent::new(),
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), ApplicationError> {
        self.render(terminal)?;

        while !self.state.should_quit {
            let application_event = self.event_broker.next_event().await?;
            self.update(application_event);

            loop {
                let event_res = self.event_broker.try_next_event();
                match event_res {
                    Ok(event) => {
                        self.update(event);
                        if self.state.should_quit {
                            break;
                        }
                    }
                    Err(e) => match e {
                        ApplicationError::EventChannelEmpty => break,
                        _ => {
                            self.state.should_quit = true;
                            return Err(e);
                        }
                    },
                }
            }

            self.render(terminal)?;
        }

        Ok(())
    }

    fn render(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), ApplicationError> {
        if !self.state.should_quit {
            terminal.draw(|frame| {
                let area = frame.area();
                self.active_view.draw(&self.state, frame, area);
                if self.state.ui.show_event_overlay {
                    self.event_overlay.draw(&self.state, frame, area);
                }
                self.notification_overlay.draw(&self.state, frame, area);
            })?;
        }

        Ok(())
    }

    fn push_event(&mut self, title: &str, text: String) {
        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
        self.state.ui.event_history.push(format!(
            "[{}] [{}] {}",
            time_str,
            title.to_uppercase(),
            text
        ));

        let len = self.state.ui.event_history.len();
        if len > MAX_EVENT_HISTORY {
            self.state
                .ui
                .event_history
                .drain(0..(len - MAX_EVENT_HISTORY));
        }
    }

    fn update(&mut self, event: ApplicationEvent) {
        match event {
            ApplicationEvent::Tick => self.handle_tick(),
            ApplicationEvent::Terminal(crossterm_event) => {
                self.handle_terminal_event(crossterm_event);
            }
            ApplicationEvent::Network(network_event) => {
                self.handle_network_event(network_event);
            }
            ApplicationEvent::Api(api_event) => self.handle_api_event(api_event),
            ApplicationEvent::SendRequest(request) => self.handle_request(request),
            ApplicationEvent::SendRawCommand(command) => self.handle_raw_command(command),
        }
    }

    pub(crate) fn handle_tick(&mut self) {
        self.state
            .ui
            .notifications
            .retain(|n| Instant::now() < n.expires_at);

        self.active_view.on_tick(&mut self.state);
    }

    fn handle_raw_command(&mut self, command: String) {
        if let Some(request) = ApiRequest::parse(&command) {
            self.push_event("user input", command);

            if let Some(network_manager) = &self.network_manager {
                let envelope = RequestEnvelope::new(request);
                network_manager.send_command(envelope);
            }
        } else {
            self.push_event(
                "user input",
                format!("Unknown or invalid command: {}", command),
            );

            self.state.ui.push(Notification::warning(format!(
                "Unknown or invalid command: {}",
                command
            )));
        }
    }

    fn handle_request(&mut self, request: ApiRequest) {
        self.push_event("request", format!("{:?}", request));

        if let Some(network_manager) = &self.network_manager {
            let envelope = RequestEnvelope::new(request);
            network_manager.send_command(envelope);
        }
    }
}
