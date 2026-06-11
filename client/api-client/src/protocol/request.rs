use crate::protocol::response::ServerResponse;
use tokio::sync::oneshot;

pub struct Request {
    pub raw_command: String,
    pub reply_to: oneshot::Sender<ServerResponse>,
}

impl Request {
    pub fn new(command: String) -> (Self, oneshot::Receiver<ServerResponse>) {
        let (transmitter, receiver) = oneshot::channel::<ServerResponse>();
        (
            Request {
                raw_command: command,
                reply_to: transmitter,
            },
            receiver,
        )
    }

    pub fn handshake() -> (Self, oneshot::Receiver<ServerResponse>) {
        Self::new(String::new())
    }
}
