mod handlers;

use crate::events::{ApplicationEvent, CustomEvent, EventBroker};
use crate::manifest::Manifest;
use crate::network::{NetworkManager, RequestChain};
use crate::notification::{Notification, NotificationTopic};
use crate::renderer::ViewManager;
use crate::renderer::components::Component;
use crate::renderer::views::LoginView;
use crate::states::AppState;
use crate::{Assets, ClientError};
use client_api::ApiRequest;
use client_api::commands::{
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
            event_broker: EventBroker::new(),
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
        self.send_chain(RequestChain::build(request));
    }

    pub fn send_chain(&mut self, chain: RequestChain) {
        let Some(network_manager) = &self.network_manager else {
            self.record_trace("dropped request", format!("{:?}: not connected", chain));
            return;
        };

        if let Err(chain) = network_manager.send_command(chain) {
            let message = format!("{:?}: the command queue is full", chain);

            self.record_trace("dropped request", message.clone());

            self.state
                .ui
                .notifications
                .push(Notification::warning(message).with_topic(NotificationTopic::Protocol))
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
        self.send_chain(RequestChain::new(vec![
            ApiRequest::Who(WhoCommand),
            ApiRequest::Status(StatusCommand),
            ApiRequest::Inventory(InventoryCommand),
            ApiRequest::Quests(QuestsCommand),
            ApiRequest::Look(LookCommand),
        ]));
    }

    pub fn update(&mut self, event: ApplicationEvent) {
        match event {
            ApplicationEvent::Tick => self.handle_tick(),
            ApplicationEvent::DeviceEvent(crossterm_event) => {
                self.handle_device_event(crossterm_event);
            }
            ApplicationEvent::Connection(event) => {
                self.handle_connection_event(event);
            }
            ApplicationEvent::Api(event) => self.handle_api_event(event),
            ApplicationEvent::Send(event) => self.handle_send_event(event),
            ApplicationEvent::Custom(event) => match event {
                CustomEvent::FightTimedOut => self.on_fight_timed_out(),
                CustomEvent::Lag(has_lag) => {
                    self.state.network.has_lag = has_lag;
                }
            },
        }
    }

    pub fn disconnect(&mut self) {
        self.network_manager = None;

        self.view_manager.set_view(Box::new(LoginView::new(
            self.state.network.server_ip.clone(),
            self.state.network.server_port.clone(),
        )));

        self.state = AppState::new(
            self.state.network.server_ip.clone(),
            self.state.network.server_port.clone(),
            self.state.game.manifest.clone(),
            self.state.game.assets.clone(),
        );
    }
}
