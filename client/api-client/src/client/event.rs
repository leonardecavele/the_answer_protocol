use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct SpawnData {
    pub r#type: String,
    pub id: String,
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
    Disconnected,
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
    ConnectionLost(String),
}

impl From<ServerResponse> for ServerEvent {
    fn from(response: ServerResponse) -> Self {
        let args: Vec<&str> = response.arguments.iter().map(|s| s.as_str()).collect();

        match args.as_slice() {
            ["CONNECT", name] => ServerEvent::Connect(name.to_string()),

            ["GAME", "SERVER", status] => match status.to_uppercase().as_str() {
                "CONNECTED" => ServerEvent::GameServer(GameServerEvent::Connected),
                "DISCONNECTED" => ServerEvent::GameServer(GameServerEvent::Disconnected),
                _ => ServerEvent::Unknown(status.to_string()),
            },

            ["SPAWN", r#type, id] => {
                let arg_type = r#type
                    .strip_prefix("type=")
                    .and_then(|s| s.parse::<String>().ok());
                let arg_id = id
                    .strip_prefix("id=")
                    .and_then(|s| s.parse::<String>().ok());

                if let Some(v_type) = arg_type
                    && let Some(v_id) = arg_id
                {
                    ServerEvent::Spawn(SpawnData {
                        r#type: v_type.to_uppercase(),
                        id: v_id,
                    })
                } else {
                    ServerEvent::Unknown(args.join(" "))
                }
            }
            ["DESPAWN", r#type, id] => {
                let arg_type = r#type
                    .strip_prefix("type=")
                    .and_then(|s| s.parse::<String>().ok());
                let arg_id = id
                    .strip_prefix("id=")
                    .and_then(|s| s.parse::<String>().ok());

                if let Some(v_type) = arg_type
                    && let Some(v_id) = arg_id
                {
                    ServerEvent::Despawn(SpawnData {
                        r#type: v_type.to_uppercase(),
                        id: v_id,
                    })
                } else {
                    ServerEvent::Unknown(args.join(" "))
                }
            }

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
