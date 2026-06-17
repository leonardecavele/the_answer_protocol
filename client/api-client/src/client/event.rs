use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{debug, warn};
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum RoomEvent {
    PresenceEnter(String),
    PresenceLeave(String),
    Chat(ChatMessage),
}

#[derive(Debug, Clone)]
pub enum GroupEvent {
    Invite(String),
    Join(String),
    Leave(String),
    Chat(ChatMessage),
}

#[derive(Debug, Clone)]
pub enum ServerEvent {
    Connect(String),
    Quit(String),
    Room(RoomEvent),
    Group(GroupEvent),
    GlobalChat(ChatMessage),
    PrivateChat(ChatMessage),
    Stats(u32),
    Unknown(String),
}

impl From<ServerResponse> for ServerEvent {
    fn from(response: ServerResponse) -> Self {
        let args: Vec<&str> = response.arguments.iter().map(|s| s.as_str()).collect();

        match args.as_slice() {
            ["CONNECT", name] => ServerEvent::Connect(name.to_string()),
            ["QUIT", name] => ServerEvent::Quit(name.to_string()),

            // Room events
            ["ROOM", "PRESENCE", "ENTER", name] => {
                ServerEvent::Room(RoomEvent::PresenceEnter(name.to_string()))
            }
            ["ROOM", "PRESENCE", "LEAVE", name] => {
                ServerEvent::Room(RoomEvent::PresenceLeave(name.to_string()))
            }
            ["ROOM", "CHAT", sender, message @ ..] => {
                ServerEvent::Room(RoomEvent::Chat(ChatMessage {
                    sender: sender.to_string(),
                    message: message.join(" "),
                }))
            }

            // Global events
            ["GLOBAL", "CHAT", sender, message @ ..] => {
                ServerEvent::GlobalChat(ChatMessage {
                    sender: sender.to_string(),
                    message: message.join(" "),
                })
            }

            // Private events
            ["PRIVATE", "CHAT", sender, message @ ..] => {
                ServerEvent::PrivateChat(ChatMessage {
                    sender: sender.to_string(),
                    message: message.join(" "),
                })
            }

            // Group events
            ["GROUP", "INVITE", leader] => {
                ServerEvent::Group(GroupEvent::Invite(leader.to_string()))
            }
            ["GROUP", "JOIN", user] => {
                ServerEvent::Group(GroupEvent::Join(user.to_string()))
            }
            ["GROUP", "LEAVE", user, ..] => {
                ServerEvent::Group(GroupEvent::Leave(user.to_string()))
            }
            ["GROUP", "CHAT", sender, message @ ..] => {
                ServerEvent::Group(GroupEvent::Chat(ChatMessage {
                    sender: sender.to_string(),
                    message: message.join(" "),
                }))
            }

            // Stats events
            ["STATS", players_str] => {
                if let Some(count) = players_str
                    .strip_prefix("players=")
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    ServerEvent::Stats(count)
                } else {
                    ServerEvent::Unknown(args.join(" "))
                }
            }

            _ => ServerEvent::Unknown(args.join(" ")),
        }
    }
}

pub struct EventDispatcher {
    broadcast_sender: broadcast::Sender<ServerEvent>,
    subscriber_tasks: Vec<JoinHandle<()>>,
}

impl EventDispatcher {
    pub fn new(broadcast_sender: broadcast::Sender<ServerEvent>) -> Self {
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
                    Ok(event) => handler(event),
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("lag.. {} events dropped", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        self.subscriber_tasks.push(task);
    }
}

impl Drop for EventDispatcher {
    fn drop(&mut self) {
        debug!("dropping EventDispatcher");
        for task in self.subscriber_tasks.drain(..) {
            task.abort();
        }
    }
}
