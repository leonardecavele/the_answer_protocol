use crate::app::App;
use crate::events::NetworkEvent;
use tracing::info;

impl App {
    pub(crate) fn handle_network_event(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::ConnectionAttemptStarted {
                server_ip,
                server_port,
                player_name,
            } => {
                self.network_manager = None;

                self.network_manager = Some(crate::network::NetworkManager::start(
                    self.event_broker.sender(),
                    server_ip,
                    server_port,
                    player_name,
                ));
            }
            NetworkEvent::ConnectionEstablished {
                server_ip,
                server_port,
                player_name,
            } => {
                if let Some(network_manager) = &self.network_manager {
                    let req = api_client::protocol::command::enums::ApiRequest::Who(
                        api_client::protocol::command::core::who::WhoCommand,
                    );
                    network_manager
                        .send_command(crate::network::envelopes::RequestEnvelope::new(req));

                    let req = api_client::protocol::command::enums::ApiRequest::Status(
                        api_client::protocol::command::resource_interaction::status::StatusCommand,
                    );
                    network_manager
                        .send_command(crate::network::envelopes::RequestEnvelope::new(req));
                }

                self.state
                    .ui
                    .remove_notification(crate::network::manager::NOTIF_ID_CONNECTION_ATTEMPT);

                self.state.ui.push(crate::states::ui::Notification::info(
                    "Connected to the server successfully!",
                ));

                self.state.network.server_ip = server_ip;
                self.state.network.server_port = server_port;
                self.state.game.player_name = Some(player_name);

                self.active_view = Box::new(crate::ui::views::game::GameView::new());
            }
            NetworkEvent::ConnectionFailed { error_message } => {
                self.network_manager = None;
                self.state
                    .ui
                    .remove_notification(crate::network::manager::NOTIF_ID_CONNECTION_ATTEMPT);

                self.state
                    .ui
                    .push(crate::states::ui::Notification::error(format!(
                        "Connection failed: {}",
                        error_message
                    )));
            }
            NetworkEvent::ConnectionLost { reason } => {
                info!("Connection lost: {}", reason);
            }
            NetworkEvent::ServerPayloadReceived(server_event) => {
                self.handle_server_event(server_event);
            }
        }
    }
}
