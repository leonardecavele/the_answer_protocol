use crate::error::TapError;
use crate::protocol::response::ServerResponse;
use tokio::sync::oneshot;

pub type RequestResult = Result<ServerResponse, TapError>;

pub struct Request {
    pub raw_command: String,
    pub reply_to: oneshot::Sender<RequestResult>,
}

impl Request {
    pub fn new(command: String) -> (Self, oneshot::Receiver<RequestResult>) {
        let (transmitter, receiver) = oneshot::channel::<RequestResult>();
        (
            Request {
                raw_command: command,
                reply_to: transmitter,
            },
            receiver,
        )
    }

    pub fn handshake() -> (Self, oneshot::Receiver<RequestResult>) {
        Self::new(String::new())
    }
}
