pub struct NetworkState {
    pub server_ip: String,
    pub server_port: String,
    pub is_connected: bool,
}

impl NetworkState {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            server_ip: ip,
            server_port: port,
            is_connected: false,
        }
    }
}
