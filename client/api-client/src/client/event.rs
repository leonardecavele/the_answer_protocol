use crate::client::{Client, event};
use crate::protocol::response::ServerResponse;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct SpawnData {
    pub r#type: String,
    pub id: String
}

#[derive(Debug, Clone)]
pub enum RoomEvent {
    PresenceEnter(String),
    PresenceLeave(String),
    Chat(ChatMessage),
    Take(String, String),
    Drop(String, String),
}

#[derive(Debug, Clone)]
pub enum GroupEvent {
    Invite(String),
    Join(String),
    Leave(String),
    Chat(ChatMessage),
    Move(String),
}

#[derive(Debug, Clone)]
pub enum GameServerEvent {
    Connected,
    Disconnected
}

#[derive(Debug, Clone)]
pub enum ServerEvent {
    Connect(String),
    GameServer(GameServerEvent),
    Spawn(SpawnData),
    Despawn(SpawnData),
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

            ["GAME", "SERVER", status] => {
                match status.to_uppercase().as_str() {
                    "CONNECTED" => ServerEvent::GameServer(GameServerEvent::Connected),
                    "DISCONNECTED" => ServerEvent::GameServer(GameServerEvent::Disconnected),
                    _ => ServerEvent::Unknown(status.to_string()),
                }
            }

            ["SPAWN", r#type, id] => {
                let arg_type = r#type
                    .strip_prefix("type=")
                    .and_then(|s| s.parse::<String>().ok());
                let arg_id = id
                    .strip_prefix("id=")
                    .and_then(|s| s.parse::<String>().ok());

                if let Some(v_type) = arg_type && let Some(v_id) = arg_id {
                    ServerEvent::Spawn(SpawnData {
                        r#type: v_type.to_uppercase(),
                        id: v_id
                    })
                } else {
                    ServerEvent::Unknown(args.join(" "))
                }
            },
            ["DESPAWN", r#type, id] => {
                let arg_type = r#type
                    .strip_prefix("type=")
                    .and_then(|s| s.parse::<String>().ok());
                let arg_id = id
                    .strip_prefix("id=")
                    .and_then(|s| s.parse::<String>().ok());

                if let Some(v_type) = arg_type && let Some(v_id) = arg_id {
                    ServerEvent::Despawn(SpawnData {
                        r#type: v_type.to_uppercase(),
                        id: v_id
                    })
                } else {
                    ServerEvent::Unknown(args.join(" "))
                }
            },

            ["QUIT", name] => ServerEvent::Quit(name.to_string()),

            // Room events
            ["ROOM", name, "ENTER"] => {
                ServerEvent::Room(RoomEvent::PresenceEnter(name.to_string()))
            }
            ["ROOM", name, "LEAVE"] => {
                ServerEvent::Room(RoomEvent::PresenceLeave(name.to_string()))
            }

            ["ROOM", "CHAT", sender, message @ ..] => {
                ServerEvent::Room(RoomEvent::Chat(ChatMessage {
                    sender: sender.to_string(),
                    message: message.join(" "),
                }))
            }

            ["TAKE", player, item @ ..] => {
                ServerEvent::Room(RoomEvent::Take(player.to_string(), item.join(" ")))
            }

            ["DROP", player, item @ ..] => {
                ServerEvent::Room(RoomEvent::Drop(player.to_string(), item.join(" ")))
            }

            // Global events
            ["GLOBAL", "CHAT", sender, message @ ..] => ServerEvent::GlobalChat(ChatMessage {
                sender: sender.to_string(),
                message: message.join(" "),
            }),

            // Private events
            ["PRIVATE", "CHAT", sender, message @ ..] => ServerEvent::PrivateChat(ChatMessage {
                sender: sender.to_string(),
                message: message.join(" "),
            }),

            // Group events
            ["GROUP", "INVITE", leader] => {
                ServerEvent::Group(GroupEvent::Invite(leader.to_string()))
            }
            ["GROUP", "JOIN", user] => ServerEvent::Group(GroupEvent::Join(user.to_string())),
            ["GROUP", "LEAVE", user, ..] => ServerEvent::Group(GroupEvent::Leave(user.to_string())),
            ["GROUP", "CHAT", sender, message @ ..] => {
                ServerEvent::Group(GroupEvent::Chat(ChatMessage {
                    sender: sender.to_string(),
                    message: message.join(" "),
                }))
            }
            ["GROUPMOVE", _, direction] => {
                ServerEvent::Group(GroupEvent::Move(direction.to_string()))
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

impl Client {
    pub fn on_connect_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::Connect(name) = event {
                handler(name);
            }
        });
    }

    pub fn on_quit_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::Quit(name) = event {
                handler(name);
            }
        });
    }

    pub fn on_room_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(event::RoomEvent) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::Room(data) = event {
                handler(data);
            }
        });
    }

    pub fn on_group_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(event::GroupEvent) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::Group(data) = event {
                handler(data);
            }
        });
    }

    pub fn on_global_chat_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(event::ChatMessage) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::GlobalChat(data) = event {
                handler(data);
            }
        });
    }

    pub fn on_private_chat_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(event::ChatMessage) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::PrivateChat(data) = event {
                handler(data);
            }
        });
    }

    pub fn on_stats_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(u32) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::Stats(count) = event {
                handler(count);
            }
        });
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
