use api_client::client::event::ServerEvent;
use crossterm::event::Event as CrosstermEvent;

/// The main event enum that encapsulates all possible events in the application.
#[derive(Debug, Clone)]
pub enum ApplicationEvent {
    Terminal(CrosstermEvent),
    Tick,
    System(SystemEvent),
    Network(NetworkEvent),
    Game(GameEvent),
    UserInterface(UserInterfaceEvent),
}

/// Events related to the application lifecycle and system level actions.
#[derive(Debug, Clone)]
pub enum SystemEvent {
    QuitRequested,
}

/// Events strictly related to the network layer status and data.
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    ConnectionAttemptStarted { server_address: String },
    ConnectionEstablished,
    ConnectionFailed { error_message: String },
    ConnectionLost { reason: String },
    /// Raw payload from the API client before processing
    ServerPayloadReceived(ServerEvent),
}

/// Events representing high-level game logic and state updates.
#[derive(Debug, Clone)]
pub enum GameEvent {
    PlayerJoined { player_name: String },
    PlayerLeft { player_name: String },
    RoomContextUpdated { room_id: String, display_name: String },
    ChatMessageReceived { sender: String, message: String },
}

/// Events for UI-specific triggers (notifications, popups).
#[derive(Debug, Clone)]
pub enum UserInterfaceEvent {
    ShowNotification { 
        id: Option<String>,
        message: String, 
        notification_type: NotificationType, 
        duration_ticks: u32 
    },
    HideNotification,
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationType {
    Information,
    Warning,
    Error,
}
