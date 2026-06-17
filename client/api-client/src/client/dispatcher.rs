use crate::client::Client;
use crate::error::CommandError;
use crate::protocol::command::look::LookResponse;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::string::ToString;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

pub struct EventDispatcher {
    broadcast_sender: broadcast::Sender<ServerResponse>,
    subscriber_tasks: Vec<JoinHandle<()>>,
}

// #[derive(Debug, Deserialize, Serialize, PartialEq)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// #[serde(tag = "event_name")]
// enum EventType {
//     NewPlayer,
// }
//
// #[serde_as]
// #[derive(Debug, Deserialize, Serialize)]
// pub struct Event {
//     player: String,
//     #[serde(flatten)]
//     event_type: EventType,
//     data: HashMap<String, Value>
// }

#[derive(Debug)]
pub enum ServerEvent {
    Connect(String),
    Quit(String),
    GlobalChat(GlobalChatEvent),
    PrivateChat(PrivateChatEvent),
    RoomPresence(RoomPresenceEvent),
    Unknown(String),
}

#[derive(Debug)]
pub struct GlobalChatEvent {
    pub sender: String,
    pub message: String,
}

#[derive(Debug)]
pub struct PrivateChatEvent {
    pub sender: String,
    pub message: String,
}

// Data for a room presence event
#[derive(Debug)]
pub enum RoomPresenceAction {
    Enter,
    Leave,
}

#[derive(Debug)]
pub struct RoomPresenceEvent {
    pub action: RoomPresenceAction,
}

fn parse_event(response: ServerResponse) -> ServerEvent {
    let arguments = response
        .arguments
        .iter()
        .map(|s| s as &str)
        .collect::<Vec<&str>>();

    match arguments.as_slice() {
        ["CONNECT", name] => ServerEvent::Connect(name.to_string()),
        ["QUIT", name] => ServerEvent::Quit(name.to_string()),
        ["GLOBAL", "CHAT", sender, message @ ..] => ServerEvent::GlobalChat(GlobalChatEvent {
            sender: sender.to_string(),
            message: message.join(" "),
        }),
        ["PRIVATE", "CHAT", sender, message @ ..] => ServerEvent::PrivateChat(PrivateChatEvent {
            sender: sender.to_string(),
            message: message.join(" "),
        }),
        ["ROOM", "PRESENCE", "ENTER"] => ServerEvent::RoomPresence(RoomPresenceEvent {
            action: RoomPresenceAction::Enter,
        }),
        ["ROOM", "PRESENCE", "LEAVE"] => ServerEvent::RoomPresence(RoomPresenceEvent {
            action: RoomPresenceAction::Leave,
        }),
        _ => ServerEvent::Unknown(arguments.join(" ")),
    }
}

impl EventDispatcher {
    pub fn new(broadcast_sender: broadcast::Sender<ServerResponse>) -> Self {
        Self {
            broadcast_sender,
            subscriber_tasks: vec![],
        }
    }

    pub fn subscribe<F>(&mut self, mut handler: F)
    where
        F: FnMut(ServerEvent) + Send + 'static,
    {
        let mut subscriber = self.broadcast_sender.subscribe();

        let task = tokio::spawn(async move {
            loop {
                match subscriber.recv().await {
                    Ok(response) => {
                        let event: ServerEvent = parse_event(response);
                        handler(event)
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("lag.. {} events dropped", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        self.subscriber_tasks.push(task);
    }

    pub fn shutdown(&mut self) {
        for task in self.subscriber_tasks.drain(..) {
            task.abort();
        }
    }
}
