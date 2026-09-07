use crate::protocol::response::ServerResponse;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Helper to decode a JSON event payload, logging the reason when it cannot be read.
fn parse_payload<T: DeserializeOwned>(label: &str, payload: &str) -> Option<T> {
    match serde_json::from_str(payload) {
        Ok(data) => Some(data),
        Err(error) => {
            warn!("unreadable {} payload: {}: {}", label, error, payload);
            None
        }
    }
}

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
pub struct DeathData {
    pub player_name: String,
    pub respawn_room_id: String,
}

#[derive(Debug, Clone)]
pub struct KillData {
    pub player: String,
    pub npc_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuestCompleteData {
    pub name: String,
    pub reward_items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuestStepData {
    pub name: String,
    pub current_step: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FightStartData {
    pub code: String,
    pub time: u64,
    pub nl_sep: String,
    pub sp_sep: String,
    pub npc_id: String,
    pub npc_hp: u64,
    pub npc_max_hp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FightResultData {
    pub player_name: String,
    pub success: bool,
    pub damage_dealt: u32,
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
    InviteRemoved(String),
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
pub enum QuestEvent {
    Complete(QuestCompleteData),
    Step(QuestStepData),
}

#[derive(Debug, Clone)]
pub enum ServerEvent {
    Connect(String),
    GameServer(GameServerEvent),
    Spawn(SpawnData),
    Despawn(SpawnData),
    Kill(KillData),
    Death(DeathData),
    FightStart(FightStartData),
    FightResult(FightResultData),
    FightEnd,
    Quit(String),
    Room(RoomEvent),
    Group(GroupEvent),
    GlobalChat(ChatMessage),
    PrivateChat(ChatMessage),
    Stats(u32),
    Quest(QuestEvent),
    Teleport,
    Unknown(String),
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

            ["KILL", player_name, npc_id] => ServerEvent::Kill(KillData {
                player: player_name.to_string(),
                npc_id: npc_id.to_string(),
            }),
            ["DEATH", player_name, respawn_room] => {
                let arg_respawn_room_id = respawn_room
                    .strip_prefix("respawn_room_id=")
                    .and_then(|s| s.parse::<String>().ok());

                if let Some(respawn_room_id) = arg_respawn_room_id {
                    ServerEvent::Death(DeathData {
                        player_name: player_name.to_string(),
                        respawn_room_id: respawn_room_id.to_string(),
                    })
                } else {
                    ServerEvent::Unknown(args.join(" "))
                }
            }

            ["FIGHT", "START", args @ ..] => {
                let payload = args.join(" ");

                match parse_payload::<FightStartData>("FIGHT START", &payload) {
                    Some(data) => ServerEvent::FightStart(data),
                    None => ServerEvent::Unknown(payload),
                }
            }
            ["FIGHT", "RESULT", args @ ..] => {
                let payload = args.join(" ");

                match parse_payload::<FightResultData>("FIGHT RESULT", &payload) {
                    Some(data) => ServerEvent::FightResult(data),
                    None => ServerEvent::Unknown(payload),
                }
            }
            ["FIGHT", "END"] => ServerEvent::FightEnd,

            // Room events
            ["ROOM", name, "PRESENCE", "ENTER"] => {
                ServerEvent::Room(RoomEvent::PresenceEnter(name.to_string()))
            }
            ["ROOM", name, "PRESENCE", "LEAVE"] => {
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
            ["GROUP", "INVITE", leader, "REMOVED"] => {
                ServerEvent::Group(GroupEvent::InviteRemoved(leader.to_string()))
            }
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

            ["QUEST", "COMPLETE", args @ ..] => {
                let payload = args.join(" ");

                match parse_payload::<QuestCompleteData>("QUEST COMPLETE", &payload) {
                    Some(data) => ServerEvent::Quest(QuestEvent::Complete(data)),
                    None => ServerEvent::Unknown(payload),
                }
            }

            ["QUEST", "STEP", args @ ..] => {
                let payload = args.join(" ");

                match parse_payload::<QuestStepData>("QUEST STEP", &payload) {
                    Some(data) => ServerEvent::Quest(QuestEvent::Step(data)),
                    None => ServerEvent::Unknown(payload),
                }
            }

            ["TELEPORT"] => ServerEvent::Teleport,
            ["QUIT", name] => ServerEvent::Quit(name.to_string()),

            _ => ServerEvent::Unknown(args.join(" ")),
        }
    }
}
