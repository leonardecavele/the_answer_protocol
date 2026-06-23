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

    /// The main application loop.
    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), ApplicationError> {
        // Start the network manager, giving it a clone of the event sender
        let event_sender = self.event_broker.sender();
        self.network_manager = Some(NetworkManager::start(
            event_sender,
            self.state.network.server_ip.clone(),
            self.state.network.server_port.clone(),
        ));

        while !self.state.should_quit {
            // 1. Draw the UI
            terminal.draw(|frame| {
                let area = frame.area();
                self.active_view.draw(&self.state, frame, area);
                self.event_overlay.draw(&self.state, frame, area);
                self.notification_overlay.draw(&self.state, frame, area);
            })?;

            // 2. Wait for the next event asynchronously
            let application_event = self.event_broker.next_event().await?;

            // 3. Update the state based on the event
            self.update(application_event);
        }

        Ok(())
    }

    /// Centralized update method.
    /// Determines how the application state changes in response to an event.
    fn update(&mut self, event: ApplicationEvent) {
        // Archiving the event
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
                // Handle periodic updates (animations, timeouts, etc.)
                let now = std::time::Instant::now();
                self.state.ui.notifications.retain(|n| now < n.expires_at);
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
            // Global keybinds
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

        // Pass event to global overlays first. If consumed, stop propagation.
        if self.event_overlay.is_blocking(&self.state) {
            let _ = self.event_overlay.handle_event(&mut self.state, &event);
            return;
        }

        // Notifications only intercept clicks that target them
        if let CrosstermEvent::Mouse(mouse_event) = event {
            if self.notification_overlay.is_mouse_over(mouse_event.column, mouse_event.row) {
                if self.notification_overlay.handle_event(&mut self.state, &event) {
                    return;
                }
            }
        }

        // Route event to active view
        self.active_view.handle_event(&mut self.state, &event);
    }

    fn handle_network_event(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::ConnectionAttemptStarted { server_address } => {
                info!("Attempting to connect to {}", server_address);
            }
            NetworkEvent::ConnectionEstablished => {
                info!("Connection established!");
            }
            NetworkEvent::ConnectionFailed { error_message } => {
                info!("Connection failed: {}", error_message);
                self.state.ui.notifications.push(crate::states::ui::Notification::new(
                    None,
                    format!("Connection Failed: {}", error_message),
                    crate::events::types::NotificationType::Error,
                    5000,
                ));
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
        // Game logic updates
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
