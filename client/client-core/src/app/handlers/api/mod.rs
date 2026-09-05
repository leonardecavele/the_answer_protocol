mod chat;
mod combat;
mod dialogue;
mod group;
mod player;
mod room;
mod server;

use crate::app::runtime::App;
use crate::events::ApiEvent;
use crate::network::ResponseEnvelope;
use crate::notification::Notification;
use crate::states::game::ChatChannel;
use client_api::events::{GameServerEvent, GroupEvent, RoomEvent, ServerEvent};
use client_api::{ApiRequest, ApiResponse, FrameDirection};

impl App {
    pub fn handle_api_event(&mut self, api_event: ApiEvent) {
        match api_event {
            ApiEvent::Server(server_event) => {
                self.handle_server_event(server_event);
            }
            ApiEvent::Lagged { stream, count } => {
                let message = format!("{} {}(s) lost", count, stream);
                self.record_trace("lag", message);
            }
            ApiEvent::ApiResponse(envelope) => {
                self.handle_api_response(envelope);
            }
            ApiEvent::Frame(frame) => match frame.direction {
                FrameDirection::Received => self.record_trace("frame recv", frame.line),
                FrameDirection::Sent => self.record_trace("frame sent", frame.line),
            },
        }
    }

    pub fn handle_api_response(&mut self, envelope: ResponseEnvelope) {
        if let Some(error) = envelope.response.get_error() {
            self.state
                .ui
                .notifications
                .push(Notification::warning(error.to_string()));

            return;
        }

        match (envelope.original_request, envelope.response) {
            (ApiRequest::Connect(_), ApiResponse::Connect(Ok(response))) => {
                self.on_connected(response);
            }
            (ApiRequest::Who(_), ApiResponse::Who(Ok(response))) => {
                self.on_who(response);
            }
            (ApiRequest::FightCreate(_), ApiResponse::FightCreate(Ok(_))) => {}
            (ApiRequest::FightAttack(_), ApiResponse::FightAttack(Ok(_))) => {}
            (ApiRequest::Status(_), ApiResponse::Status(Ok(response))) => {
                self.on_status(response);
            }
            (ApiRequest::GroupCreate(_), ApiResponse::GroupCreate(Ok(response))) => {
                self.on_group_created(response);
            }
            (ApiRequest::GroupJoin(cmd), ApiResponse::GroupJoin(Ok(response))) => {
                self.on_group_joined(response, cmd.leader_name);
            }
            (ApiRequest::GroupLeave(_), ApiResponse::GroupLeave(Ok(_))) => {
                self.on_group_left();
            }
            (ApiRequest::GlobalChat(cmd), ApiResponse::GlobalChat(Ok(_))) => {
                self.on_global_chat_sent(cmd.message);
            }
            (ApiRequest::RoomChat(cmd), ApiResponse::RoomChat(Ok(_))) => {
                self.on_room_chat_sent(cmd.message);
            }
            (ApiRequest::GroupChat(cmd), ApiResponse::GroupChat(Ok(_))) => {
                self.on_group_chat_sent(cmd.message);
            }
            (ApiRequest::PrivateChat(cmd), ApiResponse::PrivateChat(Ok(_))) => {
                self.on_private_chat_sent(cmd.to, cmd.message);
            }
            (ApiRequest::Look(_), ApiResponse::Look(Ok(response))) => {
                self.on_look(response);
            }
            (ApiRequest::Move(cmd), ApiResponse::Move(Ok(_))) => {
                self.on_moved(cmd.direction);
            }
            (ApiRequest::Inventory(_), ApiResponse::Inventory(Ok(response))) => {
                self.on_inventory(response);
            }
            (ApiRequest::Quests(_), ApiResponse::Quests(Ok(response))) => {
                self.on_quests(response);
            }
            (ApiRequest::Quest(_), ApiResponse::Quest(Ok(response))) => {
                self.on_quest(response);
            }
            (ApiRequest::Talk(cmd), ApiResponse::Talk(Ok(response))) => {
                self.on_talked_to(response, cmd.npc_name);
            }
            (ApiRequest::Take(_), ApiResponse::Take(Ok(response))) => {
                self.on_take_item(response);
            }
            (ApiRequest::Drop(_), ApiResponse::Drop(Ok(response))) => {
                self.on_drop_item(response);
            }
            (ApiRequest::Attack(cmd), ApiResponse::Attack(Ok(response))) => {
                self.on_attacked(response, cmd.npc_name);
            }
            (ApiRequest::Quit(_), ApiResponse::Quit(Ok(_))) => {
                self.state.should_quit = true;
            }
            (request, response) => {
                self.record_trace(
                    "unpaired response",
                    format!("request: {:?} -- response: {:?}", request, response),
                );
            }
        }
    }

    pub fn handle_server_event(&mut self, event: ServerEvent) {
        match event {
            ServerEvent::Connect(name) => {
                self.on_player_joined_server(name);
            }
            ServerEvent::Spawn(spawn_data) => match spawn_data.r#type.as_str() {
                "NPC" => {
                    self.on_npc_spawned(spawn_data);
                }
                "ITEM" => {
                    self.on_item_spawned(spawn_data);
                }
                t => {
                    self.state
                        .ui
                        .notifications
                        .push(Notification::warning(format!("Unknown spawn event: {}", t)));
                }
            },
            ServerEvent::Despawn(spawn_data) => match spawn_data.r#type.as_str() {
                "ITEM" => {
                    self.on_item_despawned(spawn_data);
                }
                t => {
                    self.state
                        .ui
                        .notifications
                        .push(Notification::warning(format!(
                            "Unknown despawn event: {}",
                            t
                        )));
                }
            },
            ServerEvent::Kill(kill_data) => {
                self.on_kill(kill_data);
            }
            ServerEvent::Death(death_data) => {
                self.on_death(death_data);
            }
            ServerEvent::FightStart(fight_data) => {
                self.on_fight_start(fight_data);
            }
            ServerEvent::FightResult(fight_result) => {
                self.on_fight_result(fight_result);
            }
            ServerEvent::FightEnd => {
                self.on_fight_end();
            }
            ServerEvent::Quit(name) => {
                self.on_player_quit_server(name);
            }
            ServerEvent::Room(room_event) => match room_event {
                RoomEvent::PresenceEnter(name) => {
                    self.on_player_entered(name);
                }
                RoomEvent::PresenceLeave(name) => {
                    self.on_player_left(name);
                }
                RoomEvent::Chat(chat) => {
                    self.on_chat_received(ChatChannel::Room, chat.sender, chat.message);
                }
                RoomEvent::Take(player, item_id) => {
                    self.on_item_taken_by(player, item_id);
                }
                RoomEvent::Drop(player, item_id) => {
                    self.on_item_dropped_by(player, item_id);
                }
            },
            ServerEvent::Group(group_event) => match group_event {
                GroupEvent::Invite(leader) => {
                    self.on_group_invited_by(leader);
                }
                GroupEvent::InviteRemoved(leader) => {
                    self.on_group_invite_removed(leader);
                }
                GroupEvent::Join(user) => {
                    self.on_group_member_joined(user);
                }
                GroupEvent::Leave(user) => {
                    self.on_group_member_left(user);
                }
                GroupEvent::Chat(chat) => {
                    self.on_chat_received(ChatChannel::Group, chat.sender, chat.message);
                }
                GroupEvent::Move(direction) => {
                    self.on_group_moved(direction);
                }
            },
            ServerEvent::GlobalChat(chat) => {
                self.on_chat_received(ChatChannel::Global, chat.sender, chat.message);
            }
            ServerEvent::PrivateChat(chat) => {
                let channel = ChatChannel::Private(chat.sender.clone());
                self.on_chat_received(channel, chat.sender, chat.message);
            }
            ServerEvent::Stats(count) => {
                self.on_stats(count);
            }
            ServerEvent::Unknown(raw) => {
                self.on_unknown_event(raw);
            }
            ServerEvent::GameServer(game_server_event) => match game_server_event {
                GameServerEvent::Connected => {
                    self.on_game_server_connected();
                }
                GameServerEvent::Disconnected => {
                    self.on_game_server_disconnected();
                }
            },
        }
    }
}
