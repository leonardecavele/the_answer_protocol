mod envelopes;
mod manager;

pub use envelopes::{RequestEnvelope, ResponseEnvelope};
pub use manager::{NOTIF_ID_CONNECTION_ATTEMPT, NetworkManager};
