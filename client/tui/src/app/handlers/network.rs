use crate::app::App;
use crate::events::NetworkEvent;
use crate::network::NetworkManager;
use crate::network::envelopes::RequestEnvelope;
use crate::network::manager::NOTIF_ID_CONNECTION_ATTEMPT;
use crate::states::ui::Notification;
use crate::ui::views::game::GameView;
use api_client::protocol::command::core::who::WhoCommand;
use api_client::protocol::command::enums::ApiRequest;
use api_client::protocol::command::resource_interaction::inventory::InventoryCommand;
use api_client::protocol::command::resource_interaction::quests::QuestsCommand;
use api_client::protocol::command::resource_interaction::status::StatusCommand;
use tracing::info;

impl App {
    pub(crate) fn handle_network_event(&mut self, event: NetworkEvent) {
        self.push_event("network", format!("{:?}", event));

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
                    player_name,
                ));
            }
            NetworkEvent::ConnectionEstablished {
                server_ip,
                server_port,
                player_name,
            } => {
                if let Some(network_manager) = &self.network_manager {
                    let req = ApiRequest::Who(WhoCommand);
                    network_manager.send_command(RequestEnvelope::new(req));

                    let req = ApiRequest::Status(StatusCommand);
                    network_manager.send_command(RequestEnvelope::new(req));

                    let req_inv = ApiRequest::Inventory(InventoryCommand);
                    network_manager.send_command(RequestEnvelope::new(req_inv));

                    let req_quests = ApiRequest::Quests(QuestsCommand);
                    network_manager.send_command(RequestEnvelope::new(req_quests));
                }

                self.state
                    .ui
                    .remove_notification(NOTIF_ID_CONNECTION_ATTEMPT);

                self.state
                    .ui
                    .push(Notification::info("Connected to the server successfully!"));

                self.state.network.server_ip = server_ip;
                self.state.network.server_port = server_port;
                self.state.game.player_name = Some(player_name);

                self.active_view = Box::new(GameView::new());
            }
            NetworkEvent::ConnectionFailed { error_message } => {
                self.network_manager = None;
                self.state
                    .ui
                    .remove_notification(NOTIF_ID_CONNECTION_ATTEMPT);

                self.state.ui.push(Notification::error(format!(
                    "Connection failed: {}",
                    error_message
                )));
            }
            NetworkEvent::ConnectionLost { reason } => {
                info!("Connection lost: {}", reason);
            }
        }
    }
}
