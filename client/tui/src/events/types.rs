use crate::network::envelopes::{RequestEnvelope, ResponseEnvelope};
use api_client::client::event::ServerEvent;
use api_client::protocol::command::enums::ApiRequest;
use crossterm::event::Event as CrosstermEvent;

/// The main event enum that encapsulates all possible events in the application.
#[derive(Debug, Clone)]
pub enum ApplicationEvent {
    Terminal(CrosstermEvent),
    Tick,
    Network(NetworkEvent),
    SendRequest(ApiRequest),
    SendRawCommand(String),
    Api(ApiEvent),
}

#[derive(Debug, Clone)]
pub enum ApiEvent {
    LogApiRequest(RequestEnvelope),
    ApiResponse(ResponseEnvelope),
    Server(ServerEvent),
}

/// Events strictly related to the network layer status and data.
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    ConnectionAttemptStarted {
        server_ip: String,
        server_port: String,
        player_name: String,
    },
    ConnectionEstablished {
        server_ip: String,
        server_port: String,
        player_name: String,
    },
    ConnectionFailed {
        error_message: String,
    },
    ConnectionLost {
        reason: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationType {
    Information,
    Warning,
    Error,
}
