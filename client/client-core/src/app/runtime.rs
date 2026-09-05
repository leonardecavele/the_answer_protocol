use crate::Assets;
use crate::ClientError;
use crate::events::{ApplicationEvent, EventBroker};
use crate::manifest::Manifest;
use crate::network::NetworkManager;
use crate::network::envelopes::RequestEnvelope;
use crate::renderer::components::{Component, Lifecycle};
use crate::renderer::view::ViewManager;
use crate::states::app::AppState;
use crate::states::notification::Notification;
use api_client::ApiRequest;
use api_client::commands::{
    InventoryCommand, LookCommand, QuestsCommand, StatusCommand, WhoCommand,
};
use ratatui::Frame;
use std::sync::Arc;

pub struct App {
    pub state: AppState,
    pub event_broker: EventBroker,
    pub network_manager: Option<NetworkManager>,
    pub view_manager: ViewManager,
}

impl App {
    pub fn new(ip: String, port: String, assets: Assets) -> Self {
        Self::with_broker(ip, port, assets, EventBroker::new())
    }

    fn with_broker(ip: String, port: String, assets: Assets, event_broker: EventBroker) -> Self {
        let (manifest, err) = match Manifest::load(&assets) {
            Ok(manifest) => (manifest, None),
            Err(error) => (Manifest::default(), Some(error)),
        };

        let mut state = AppState::new(ip.clone(), port.clone(), Arc::new(manifest), assets);
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

    pub fn try_next_event(&mut self) -> Result<ApplicationEvent, ClientError> {
        self.event_broker.try_next_event()
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        self.view_manager.draw(&self.state, frame, area);
    }

    pub fn send(&mut self, request: ApiRequest) {
        if let Some(network_manager) = &self.network_manager {
            let envelope = RequestEnvelope::new(request);
            network_manager.send_command(envelope);
        }
    }

    pub fn record_trace(&mut self, title: &str, text: String) {
        let time_str = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        self.state.ui.trace_log.push(format!(
            "[{}] [{}] {}",
            time_str,
            title.to_uppercase(),
            text
        ));
    }

    pub fn load_state_from_server(&mut self) {
        self.send(ApiRequest::Who(WhoCommand));
        self.send(ApiRequest::Status(StatusCommand));
        self.send(ApiRequest::Inventory(InventoryCommand));
        self.send(ApiRequest::Quests(QuestsCommand));
        self.send(ApiRequest::Look(LookCommand));
    }

    pub fn update(&mut self, event: ApplicationEvent) {
        match event {
            ApplicationEvent::Tick => self.handle_tick(),
            ApplicationEvent::DeviceEvent(crossterm_event) => {
                self.handle_device_event(crossterm_event);
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
