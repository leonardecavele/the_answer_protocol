use crate::protocol::response::ServerResponse;

pub struct Event {
    pub forward_channel: tokio::sync::mpsc::Sender<ServerResponse>,
}