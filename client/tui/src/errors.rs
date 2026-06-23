use std::fmt;
use std::io;

/// Centralized error type for the TUI application.
#[derive(Debug)]
pub enum ApplicationError {
    /// Errors originating from standard I/O operations (e.g. terminal setup).
    Io(io::Error),
    /// Errors originating from the network layer or API client.
    Network(String),
    /// Errors originating from the event broker channel being closed or failing.
    EventChannelClosed,
    /// General application-level errors.
    General(String),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplicationError::Io(err) => write!(f, "I/O Error: {}", err),
            ApplicationError::Network(msg) => write!(f, "Network Error: {}", msg),
            ApplicationError::EventChannelClosed => write!(f, "Event channel was closed unexpectedly"),
            ApplicationError::General(msg) => write!(f, "Application Error: {}", msg),
        }
    }
}

impl std::error::Error for ApplicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ApplicationError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ApplicationError {
    fn from(error: io::Error) -> Self {
        ApplicationError::Io(error)
    }
}
