use crate::protocol::response::ServerResponse;
use tokio::sync::oneshot;

pub struct Request {
    pub command: String,
    pub forward_channel: tokio::sync::oneshot::Sender<ServerResponse>,
}

impl Request {
    pub fn new(command: String) -> (Self, tokio::sync::oneshot::Receiver<ServerResponse>) {
        let (transmitter, receiver) = oneshot::channel::<ServerResponse>();
        (
            Request {
                command,
                forward_channel: transmitter,
            },
            receiver,
        )
    }

    pub fn handshake() -> (Self, tokio::sync::oneshot::Receiver<ServerResponse>) {
        Self::new(String::new())
    }
}
