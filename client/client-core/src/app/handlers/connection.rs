use crate::app::App;
use crate::events::ConnectionEvent;
use crate::network::NetworkManager;
use crate::notification::{Notification, NotificationTopic};
use crate::renderer::views::GameView;

impl App {
    pub fn handle_connection_event(&mut self, event: ConnectionEvent) {
        self.record_trace("network", format!("{:?}", event));

        match event {
            ConnectionEvent::AttemptStarted {
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
            ConnectionEvent::Established {
                server_ip,
                server_port,
                player_name,
            } => {
                self.state.ui.notifications.push(
                    Notification::success("Connected to the server successfully!")
                        .with_topic(NotificationTopic::Connection),
                );

                self.state.network.server_ip = server_ip;
                self.state.network.server_port = server_port;
                self.state.network.is_connected = true;
                self.state.game.player.set_name(player_name);

                self.load_state_from_server();
                self.view_manager.set_view(Box::new(GameView::new()));
            }
            ConnectionEvent::Failed { error_message } => {
                self.network_manager = None;

                self.state.ui.notifications.push(
                    Notification::error(format!("Connection failed: {}", error_message))
                        .with_topic(NotificationTopic::Connection),
                );
            }
            ConnectionEvent::Lost { reason } => {
                self.disconnect();

                self.state.ui.notifications.push(
                    Notification::error(format!("Connection lost: {}", reason))
                        .with_topic(NotificationTopic::Connection),
                );
            }
        }
    }
}
