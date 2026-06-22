use api_client::client::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected(String), // player name
}

pub struct NetworkState {
    pub client: Option<Arc<Mutex<Client>>>,
    pub connection_state: ConnectionState,
    pub server_ip: String,
    pub server_port: String,
    pub connected_at: Option<std::time::Instant>,
}

impl NetworkState {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            client: None,
            connection_state: ConnectionState::Disconnected,
            server_ip: ip,
            server_port: port,
            connected_at: None,
        }
    }
}
