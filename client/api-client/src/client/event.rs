use crate::protocol::response::ServerResponse;
use std::string::ToString;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::warn;

pub struct EventDispatcher {
    broadcast_sender: broadcast::Sender<ServerResponse>,
    subscriber_tasks: Vec<JoinHandle<()>>,
}

#[derive(Debug)]
pub enum ServerEvent {
    Connect(String),
    Quit(String),
    Chat(ChatEventData),
    RoomPresence(RoomPresenceType),
    Unknown(String),
}

#[derive(Debug)]
pub enum ChatScopeType {
    Global,
    Room,
    Private,
}

#[derive(Debug)]
pub struct ChatEventData {
    pub scope: ChatScopeType,
    pub sender: String,
    pub message: String,
}

// Data for a room presence event
#[derive(Debug)]
pub enum RoomPresenceType {
    Enter,
    Leave,
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
        ["GLOBAL", "CHAT", sender, message @ ..] => ServerEvent::Chat(ChatEventData {
            scope: ChatScopeType::Global,
            sender: sender.to_string(),
            message: message.join(" "),
        }),
        ["ROOM", "CHAT", sender, message @ ..] => ServerEvent::Chat(ChatEventData {
            scope: ChatScopeType::Room,
            sender: sender.to_string(),
            message: message.join(" "),
        }),
        ["PRIVATE", "CHAT", sender, message @ ..] => ServerEvent::Chat(ChatEventData {
            scope: ChatScopeType::Private,
            sender: sender.to_string(),
            message: message.join(" "),
        }),
        ["ROOM", "PRESENCE", "ENTER"] => ServerEvent::RoomPresence(RoomPresenceType::Enter),
        ["ROOM", "PRESENCE", "LEAVE"] => ServerEvent::RoomPresence(RoomPresenceType::Leave),
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
