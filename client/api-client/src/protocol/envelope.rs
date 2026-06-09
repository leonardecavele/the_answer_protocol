use tokio::sync::oneshot::Sender;
use crate::protocol::command::Command;
use crate::protocol::packet::Packet;

pub struct Envelope {
    pub command: Command,
    pub tx: Option<Sender<Packet>>,
}
