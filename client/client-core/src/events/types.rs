use client_api::events::ServerEvent;
use client_api::{ApiRequest, ApiResponse, Frame};
use crossterm::event::Event as CrosstermEvent;

/// The main event enum that encapsulates all possible events in the application.
#[derive(Debug, Clone)]
pub enum ApplicationEvent {
    DeviceEvent(CrosstermEvent),
    Tick,
    Network(NetworkConnectionEvent),
    SendRequest(ApiRequest),
    SendRawCommand(String),
    Protocol(ProtocolEvent),
    FightTimedOut,
}

#[derive(Debug, Clone)]
pub enum ProtocolEvent {
    ApiResponse {
        response: ApiResponse,
        original_request: ApiRequest,
    },
    Server(ServerEvent),
    Frame(Frame),
    Lagged {
        stream: &'static str,
        count: usize,
    },
    RequestFailed {
        request: ApiRequest,
        error_message: String,
    },
}

/// Events strictly related to the network layer status and data.
#[derive(Debug, Clone)]
pub enum NetworkConnectionEvent {
    AttemptStarted {
        server_ip: String,
        server_port: String,
        player_name: String,
    },
    Established {
        server_ip: String,
        server_port: String,
        player_name: String,
    },
    Failed {
        error_message: String,
    },
    Lost {
        reason: String,
    },
}
