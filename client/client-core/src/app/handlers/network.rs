use crate::app::runtime::App;
use crate::events::NetworkEvent;
use crate::network::NOTIF_ID_CONNECTION_ATTEMPT;
use crate::network::NetworkManager;
use crate::renderer::views::game::GameView;
use crate::renderer::views::login::LoginView;
use crate::states::app::AppState;
use crate::states::notification::Notification;

impl App {
    pub fn handle_network_event(&mut self, event: NetworkEvent) {
        self.record_trace("network", format!("{:?}", event));

        match event {
            NetworkEvent::ConnectionAttemptStarted {
                server_ip,
                server_port,
                player_name,
            } => {
                self.network_manager = None;

                self.network_manager = Some(NetworkManager::start(
                    self.event_broker.sender(),
                    server_ip,
                    server_port,
                    player_name.to_uppercase(),
                ));
            }
            NetworkEvent::ConnectionEstablished {
                server_ip,
                server_port,
                player_name,
            } => {
                self.state
                    .ui
                    .notifications
                    .remove(NOTIF_ID_CONNECTION_ATTEMPT);

                self.state.ui.notifications.push(Notification::success(
                    "Connected to the server successfully!",
                ));

                self.state.network.server_ip = server_ip;
                self.state.network.server_port = server_port;
                self.state.network.is_connected = true;
                self.state.game.player.set_name(player_name);

                self.load_state_from_server();
                self.view_manager.set_view(Box::new(GameView::new()));
            }
            NetworkEvent::ConnectionFailed { error_message } => {
                self.network_manager = None;
                self.state
                    .ui
                    .notifications
                    .remove(NOTIF_ID_CONNECTION_ATTEMPT);

                self.state
                    .ui
                    .notifications
                    .push(Notification::error(format!(
                        "Connection failed: {}",
                        error_message
                    )));
            }
            NetworkEvent::ConnectionLost { reason } => {
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

                self.state
                    .ui
                    .notifications
                    .push(Notification::error(format!("Connection lost: {}", reason)));
            }
        }
    }
}
