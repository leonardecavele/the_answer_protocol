use crate::data::manifest::Manifest;
use crate::errors::ApplicationError;
use crate::events::{ApplicationEvent, EventBroker};
use crate::network::NetworkManager;
use crate::network::envelopes::RequestEnvelope;
use crate::states::app::AppState;
use crate::states::notification::Notification;
use crate::ui::components::{Component, Lifecycle};
use crate::ui::view::ViewManager;
use api_client::ApiRequest;
use api_client::commands::{
    InventoryCommand, LookCommand, QuestsCommand, StatusCommand, WhoCommand,
};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::sync::Arc;

pub struct App {
    pub state: AppState,
    pub(super) event_broker: EventBroker,
    pub(super) network_manager: Option<NetworkManager>,
    pub(super) view_manager: ViewManager,
}

impl App {
    pub fn new(ip: String, port: String) -> Self {
        Self::with_broker(ip, port, EventBroker::new())
    }

    pub fn with_terminal_input(ip: String, port: String) -> Self {
        Self::with_broker(ip, port, EventBroker::with_terminal_input())
    }

    fn with_broker(ip: String, port: String, event_broker: EventBroker) -> Self {
        let (manifest, err) = match Manifest::load() {
            Ok(manifest) => (manifest, None),
            Err(error) => (Manifest::default(), Some(error)),
        };

        let mut state = AppState::new(ip.clone(), port.clone(), Arc::new(manifest));
        if let Some(error) = err {
            state
                .ui
                .notifications
                .push(Notification::error(error).with_ms(10000));
        }

        Self {
            state,
            event_broker,
            network_manager: None,
            view_manager: ViewManager::new(ip, port),
        }
    }

    pub fn try_next_event(&mut self) -> Result<ApplicationEvent, ApplicationError> {
        self.event_broker.try_next_event()
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        self.view_manager.draw(&self.state, frame, area);
    }

    pub(super) fn send(&mut self, request: ApiRequest) {
        if let Some(network_manager) = &self.network_manager {
            let envelope = RequestEnvelope::new(request);
            network_manager.send_command(envelope);
        }
    }

    pub(super) fn record_trace(&mut self, title: &str, text: String) {
        let time_str = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        self.state.ui.trace_log.push(format!(
            "[{}] [{}] {}",
            time_str,
            title.to_uppercase(),
            text
        ));
    }

    pub(super) fn load_state_from_server(&mut self) {
        self.send(ApiRequest::Who(WhoCommand));
        self.send(ApiRequest::Status(StatusCommand));
        self.send(ApiRequest::Inventory(InventoryCommand));
        self.send(ApiRequest::Quests(QuestsCommand));
        self.send(ApiRequest::Look(LookCommand));
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
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    pub fn update(&mut self, event: ApplicationEvent) {
        match event {
            ApplicationEvent::Tick => self.handle_tick(),
            ApplicationEvent::Terminal(crossterm_event) => {
                self.handle_terminal_event(crossterm_event);
            }
            ApplicationEvent::Network(network_event) => {
                self.handle_network_event(network_event);
            }
            ApplicationEvent::Api(api_event) => self.handle_api_event(api_event),
            ApplicationEvent::SendRequest(request) => self.send(request),
            ApplicationEvent::SendRawCommand(command) => self.handle_raw_command(command),
            ApplicationEvent::FightTimedOut => self.on_fight_timed_out(),
        }
    }

    fn handle_tick(&mut self) {
        self.state.ui.notifications.retain_active();

        let sender = self.event_broker.sender();
        self.view_manager.on_tick(&mut self.state, &sender);
    }

    fn handle_raw_command(&mut self, command: String) {
        if let Some(request) = ApiRequest::parse(&command) {
            self.record_trace("user input", command);
            self.send(request);
        } else {
            let message = format!("Unknown or invalid command: {}", command);

            self.record_trace("user input", message.clone());

            self.state
                .ui
                .notifications
                .push(Notification::warning(message));
        }
    }
}
