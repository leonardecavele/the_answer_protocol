use api_client::client::event::ServerEvent;
use api_client::error::{CommandError, NetworkError, TapError};

pub enum NetworkCommand {
    ExecuteConnect(String),
    RequestLook,
    RequestQuit,
}

pub enum GameEvent {
    /// Update the inventory display
    InventoryUpdate(Vec<String>),
    /// Update the online players counter
    UpdateOnlinePlayers(u32),
    /// Emitted when the user successfully sends a chat message
    LocalChatSent(crate::state::ChatScope, String),
    /// Emitted when the player's group changes
    UpdateGroup(Option<String>),
    /// Emitted when moving or refreshing the room
    UpdateRoomContext {
        room_id: String,
        room_display_name: String,
        npcs: Vec<String>,
    },
    /// Emitted when HP changes
    UpdateStatus {
        hp: u32,
        max_hp: u32,
    },
    /// Emitted when a user command returns a successful response
    CommandResult(String),
    /// Emitted when a user command fails
    CommandError(CommandError),
    /// Unrecognized command
    UnknowCommand(String),
    /// Raw output line to append to game log
    PushGameOutput(String),
}

pub enum NetEvent {
    /// An event received from the game server API
    Api(ServerEvent),
    /// The API client successfully connected (TCP level)
    ClientConnected(api_client::client::Client),
    /// Trigger the CONNECT protocol command after TCP is established
    AttemptConnect(String, String, String),
    /// Trigger the LOGIN/CONNECT logic
    ExecuteConnect(String),
    /// Emitted when the initial game login succeeds
    LoginSuccess(String),
    /// The API client disconnected from the server
    ApiDisconnected,
    /// Protocol or network errors
    NetworkError(NetworkError),
    TapError(TapError),
}

pub enum UiEvent {
    /// A terminal event (key, mouse, resize)
    TerminalEvent(crossterm::event::Event),
    /// A regular tick event (useful for animations or timeouts)
    Tick,
    /// Push a notification
    Notification(String, crate::state::NotificationType, u32),
}

pub enum AppEvent {
    Game(GameEvent),
    Network(NetEvent),
    Ui(UiEvent),
}

pub mod router;
pub mod net;
pub mod game;
pub mod ui;
