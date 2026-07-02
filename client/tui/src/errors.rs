use std::io;

/// Centralized error type for the TUI application.
#[derive(thiserror::Error, Debug)]
pub enum ApplicationError {
    /// Errors originating from standard I/O operations (e.g. terminal setup).
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),

    /// Errors originating from the network layer or API client.
    #[error("Network Error: {0}")]
    Network(String),

    /// Errors originating from the event broker channel being closed or failing.
    #[error("Event channel was closed unexpectedly")]
    EventChannelClosed,

    #[error("Event channel is empty")]
    EventChannelEmpty,

    /// General application-level errors.
    #[error("Application Error: {0}")]
    General(String),
}
