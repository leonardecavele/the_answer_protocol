use crate::network::ResponseEnvelope;
use client_api::events::ServerEvent;
use client_api::{ApiRequest, Frame};
use crossterm::event::Event as CrosstermEvent;

/// The main event enum that encapsulates all possible events in the application.
#[derive(Debug, Clone)]
pub enum ApplicationEvent {
    DeviceEvent(CrosstermEvent),
    Tick,
    Network(NetworkConnectionEvent),
    SendRequest(ApiRequest),
    SendRawCommand(String),
    Api(ApiEvent),
    FightTimedOut,
}

#[derive(Debug, Clone)]
pub enum ApiEvent {
    ApiResponse(ResponseEnvelope),
    Server(ServerEvent),
    Frame(Frame),
    Lagged { stream: &'static str, count: usize },
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
