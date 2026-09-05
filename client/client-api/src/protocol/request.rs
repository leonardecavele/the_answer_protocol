use crate::error::TapError;
use crate::protocol::response::ServerResponse;
use tokio::sync::oneshot;

pub(crate) type RequestResult = Result<ServerResponse, TapError>;

#[derive(Debug, Clone, Copy)]
pub(crate) enum RequestFlow {
    Continue,
    End,
}

pub(crate) struct Request {
    pub raw_command: String,
    pub flow: RequestFlow,
    pub reply_to: oneshot::Sender<RequestResult>,
}

impl Request {
    pub(crate) fn new(
        command: String,
        flow: RequestFlow,
    ) -> (Self, oneshot::Receiver<RequestResult>) {
        let (transmitter, receiver) = oneshot::channel::<RequestResult>();
        (
            Request {
                raw_command: command,
                flow,
                reply_to: transmitter,
            },
            receiver,
        )
    }
}
