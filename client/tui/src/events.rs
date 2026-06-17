use api_client::client::event::ServerEvent;


pub enum AppEvent {
    /// An event received from the game server API
    Api(ServerEvent),
    
    /// The API client successfully connected
    ClientConnected(api_client::client::Client),

    /// Trigger the connect protocol command after TCP is established
    ExecuteConnect(String),

    /// Emitted when a user command returns a successful response
    CommandResult(String),

    /// Emitted when a user command fails
    CommandError(String),

    /// Emitted when the initial game login succeeds
    LoginSuccess(String),

    /// The API client disconnected from the server
    ApiDisconnected,
    
    /// A terminal event (key, mouse, resize)
    TerminalEvent(crossterm::event::Event),
    
    /// A regular tick event (useful for animations or timeouts)
    Tick,
}
