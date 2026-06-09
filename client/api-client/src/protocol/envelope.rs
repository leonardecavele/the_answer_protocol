use crate::protocol::command::Command;
use crate::protocol::packet::Packet;
use tokio::sync::oneshot::Sender;

pub struct Envelope {
    pub command: Command,
    pub tx: Option<Sender<Packet>>,
}
