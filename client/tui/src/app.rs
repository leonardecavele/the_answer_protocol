use crate::constants::{MAX_EVENT_HISTORY, TICK_RATE};
use crate::errors::ApplicationError;
use crate::events::{
    ApplicationEvent, EventBroker, GameEvent, NetworkEvent, SystemEvent, UserInterfaceEvent,
};
use crate::network::NetworkManager;
use crate::states::app::AppState;
use crate::ui::components::event_overlay::EventOverlayComponent;
use crate::ui::components::notifications::NotificationComponent;
use crate::ui::components::Component;
use crate::ui::views::LoginView;
use crate::ui::AppView;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Instant;
use tracing::info;

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
                self.state.ui.notifications.retain(|n| Instant::now() < n.expires_at);
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
            ApplicationEvent::Game(game_event) => {
                self.handle_game_event(game_event);
            }
            ApplicationEvent::UserInterface(ui_event) => {
                self.handle_ui_event(ui_event);
            }
        }
    }

    fn handle_terminal_event(&mut self, event: CrosstermEvent) {
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
            let _ = self.event_overlay.handle_event(&mut self.state, &event, &self.event_broker.sender());
            return;
        }

        if let CrosstermEvent::Mouse(mouse_event) = event {
            if self.notification_overlay.is_mouse_over(mouse_event.column, mouse_event.row) {
                if self.notification_overlay.handle_event(&mut self.state, &event, &self.event_broker.sender()) {
                    return;
                }
            }
        }

        self.active_view.handle_event(&mut self.state, &event, &self.event_broker.sender());
    }

    fn handle_network_event(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::ConnectionAttemptStarted { server_ip, server_port, player_name } => {
                self.network_manager = None;
                
                self.network_manager = Some(crate::network::NetworkManager::start(
                    self.event_broker.sender(),
                    server_ip,
                    server_port,
                    player_name,
                ));
            }
            NetworkEvent::ConnectionEstablished { server_ip, server_port, player_name } => {
                self.state.ui.remove_notification(crate::constants::NOTIF_ID_CONNECTION_ATTEMPT);
                
                self.state.ui.push_notification(
                    None,
                    crate::events::types::NotificationType::Information,
                    "Connected to the server successfully!".to_string(),
                    None,
                );
                
                self.state.network.server_ip = server_ip;
                self.state.network.server_port = server_port;
                self.state.game.player_name = Some(player_name);

                self.active_view = Box::new(crate::ui::views::game::GameView::new());
            }
            NetworkEvent::ConnectionFailed { error_message } => {
                self.network_manager = None;
                self.state.ui.remove_notification(crate::constants::NOTIF_ID_CONNECTION_ATTEMPT);
                
                self.state.ui.push_notification(
                    None,
                    crate::events::types::NotificationType::Error,
                    format!("Connection failed: {}", error_message),
                    None,
                );
            }
            NetworkEvent::ConnectionLost { reason } => {
                info!("Connection lost: {}", reason);
            }
            NetworkEvent::ServerPayloadReceived(_) => {
                info!("Received raw server payload");
            }
        }
    }

    fn handle_game_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::PlayerJoined { player_name } => info!("Player {} joined", player_name),
            _ => {}
        }
    }

    fn handle_ui_event(&mut self, event: UserInterfaceEvent) {
        // UI specific updates (notifications)
        match event {
            UserInterfaceEvent::ShowNotification { message, .. } => {
                info!("Notification: {}", message);
            }
            _ => {}
        }
    }
}
