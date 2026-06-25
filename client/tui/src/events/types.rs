use crate::network::envelopes::ResponseEnvelope;
use api_client::client::event::ServerEvent;
use crossterm::event::Event as CrosstermEvent;

/// The main event enum that encapsulates all possible events in the application.
#[derive(Debug, Clone)]
pub enum ApplicationEvent {
    Terminal(CrosstermEvent),
    Tick,
    System(SystemEvent),
    Network(NetworkEvent),
    ApiResponse(ResponseEnvelope),
    SendRawCommand(String),
}

/// Events related to the application lifecycle and system level actions.
#[derive(Debug, Clone)]
pub enum SystemEvent {
    QuitRequested,
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
    /// Raw payload from the API client before processing
    ServerPayloadReceived(ServerEvent),
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationType {
    Information,
    Warning,
    Error,
}
