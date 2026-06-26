use crate::errors::ApplicationError;
use crate::events::{ApplicationEvent, EventBroker};
use crate::network::NetworkManager;
use crate::states::app::AppState;
use crate::ui::components::event_overlay::EventOverlayComponent;
use crate::ui::components::notifications::NotificationComponent;
use crate::ui::components::Component;
use crate::ui::views::AppView;
use crate::ui::views::LoginView;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Instant;

pub mod handlers;

pub const MAX_EVENT_HISTORY: usize = 100;

pub struct App {
    pub state: AppState,
    pub event_broker: EventBroker,
    pub network_manager: Option<NetworkManager>,
    pub active_view: Box<dyn AppView>,
    pub event_overlay: EventOverlayComponent,
    pub notification_overlay: NotificationComponent,
}

impl App {
    pub fn new(ip: String, port: String) -> Self {
        let (manifest, err) = match crate::data::manifest::Manifest::load() {
            Ok(manifest) => (manifest, None),
            Err(error) => (crate::data::manifest::Manifest::default(), Some(error)),
        };

        let mut state = AppState::new(ip.clone(), port.clone(), manifest);
        if let Some(error) = err {
            state
                .ui
                .push(crate::states::ui::Notification::error(error).with_duration(10000));
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

    fn push_event(&mut self, title: &str, text: String) {
        self.state
            .ui
            .event_history
            .push(format!("[{}] {}", title.to_uppercase(), text));

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
        if let Some(request) = api_client::protocol::command::enums::ApiRequest::parse(&command) {
            self.push_event("user input", command);

            if let Some(network_manager) = &self.network_manager {
                let envelope = crate::network::envelopes::RequestEnvelope::new(request);
                network_manager.send_command(envelope);
            }
        } else {
            self.push_event(
                "user input",
                format!("Unknown or invalid command: {}", command),
            );

            self.state
                .ui
                .push(crate::states::ui::Notification::warning(format!(
                    "Unknown or invalid command: {}",
                    command
                )));
        }
    }
}
