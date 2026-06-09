use tokio::sync::oneshot::Sender;
use crate::command::Command;
use crate::packet::Packet;

pub struct Envelop {
    pub command: Command,
    pub tx: Option<Sender<Packet>>,
}
