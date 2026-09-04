use std::time::Duration;

pub struct ClientConfig {
    pub connect_timeout: Duration,
    pub handshake_timeout: Duration,
    pub request_timeout: Duration,
    pub close_timeout: Duration,
    pub max_frame_length: usize,
    pub command_channel_capacity: usize,
    pub event_channel_capacity: usize,
    pub frame_channel_capacity: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            connect_timeout: Duration::from_secs(5),
            handshake_timeout: Duration::from_secs(2),
            request_timeout: Duration::from_secs(10),
            close_timeout: Duration::from_secs(2),
            max_frame_length: 65_536,
            command_channel_capacity: 512,
            event_channel_capacity: 512,
            frame_channel_capacity: 512,
        }
    }
}
