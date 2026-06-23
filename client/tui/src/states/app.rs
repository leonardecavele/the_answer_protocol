
pub struct AppState {
    pub should_quit: bool,
    pub server_ip: String,
    pub server_port: String,
}

impl AppState {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            should_quit: false,
            server_ip: ip,
            server_port: port,
        }
    }
}