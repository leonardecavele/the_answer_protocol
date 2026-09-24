mod chat;
mod combat;
mod dialogue;
mod group;
mod item;
mod player;
mod quest;
mod room;
mod session;

use crate::app::App;
use crate::events::ApiEvent;
use crate::notification::{Notification, NotificationTopic};
use crate::states::game::ChatChannel;
use client_api::events::{
    ChatEvent, FightEvent, GameServerEvent, GroupEvent, ItemEvent, QuestEvent, RoomEvent,
    ServerEvent, SessionEvent,
};
use client_api::{ApiRequest, ApiResponse, ErrorCode, FrameDirection};

impl App {
    pub fn handle_api_event(&mut self, event: ApiEvent) {
        match event {
            ApiEvent::Server(server_event) => {
                self.handle_server_event(server_event);
            }
            ApiEvent::Lagged { stream, count } => {
                let message = format!("{} {}(s) lost", count, stream);
                self.record_trace("lag", message);
            }
            ApiEvent::RequestFailed {
                request,
                error_message,
            } => {
                self.record_trace(
                    "request failed",
                    format!("{:?}: {}", request, error_message),
                );

                self.state.ui.notifications.push(
                    Notification::error(format!("Command failed: {}", error_message))
                        .with_topic(NotificationTopic::Protocol),
                );
            }
            ApiEvent::ApiResponse {
                response,
                original_request,
            } => {
                self.handle_api_response(response, original_request);
            }
            ApiEvent::Frame(frame) => match frame.direction {
                FrameDirection::Received => self.record_trace("frame recv", frame.line),
                FrameDirection::Sent => self.record_trace("frame sent", frame.line),
            },
        }
    }

    pub fn handle_api_response(&mut self, response: ApiResponse, original_request: ApiRequest) {
        let response_clone = response.clone();

        if let Some(error) = response.get_error() {
            self.state
                .ui
                .notifications
                .push(Notification::warning(error.to_string()));

            if let (ApiRequest::FightAttack(_), ApiResponse::FightAttack(Err(_))) =
                (original_request, response_clone)
                && error.kind() == Some(ErrorCode::DataTooBig)
            {
                self.state.game.fight.resume()
            }

            return;
        }

        match (original_request, response) {
            (ApiRequest::Connect(_), ApiResponse::Connect(Ok(response))) => {
                self.on_connected(response);
            }
            (ApiRequest::Who(_), ApiResponse::Who(Ok(response))) => {
                self.on_who(response);
            }
            (ApiRequest::Use(_), ApiResponse::Use(Ok(response))) => {
                self.on_use(response);
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
            (ApiRequest::GroupInvite(cmd), ApiResponse::GroupInvite(Ok(_))) => {
                self.on_group_invite_sent(cmd.username);
            }
            (ApiRequest::GlobalChat(_), ApiResponse::GlobalChat(Ok(_))) => {}
            (ApiRequest::RoomChat(_), ApiResponse::RoomChat(Ok(_))) => {}
            (ApiRequest::GroupChat(_), ApiResponse::GroupChat(Ok(_))) => {}
            (ApiRequest::PrivateChat(_), ApiResponse::PrivateChat(Ok(_))) => {}
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
                self.disconnect();

                self.state.ui.notifications.push(
                    Notification::success("Successfully disconnected".to_string())
                        .with_topic(NotificationTopic::Connection),
                );
            }
            (request, response) => {
                self.record_trace(
                    "unpaired response",
                    format!("request: {:?} -- response: {:?}", request, response),
                );

                self.state.ui.notifications.push(
                    Notification::warning(
                        "The server answered a command with an unrelated response.",
                    )
                    .with_topic(NotificationTopic::Protocol),
                );
            }
        }
    }

    pub fn handle_server_event(&mut self, event: ServerEvent) {
        match event {
            ServerEvent::Session(session_event) => match session_event {
                SessionEvent::Connect(name) => self.on_player_joined_server(name),
                SessionEvent::Quit(name) => self.on_player_quit_server(name),
                SessionEvent::Stats(count) => self.on_stats(count),
                SessionEvent::Broadcast(message) => self.on_broadcast(message),
            },
            ServerEvent::GameServer(game_server_event) => match game_server_event {
                GameServerEvent::Connected => self.on_game_server_connected(),
                GameServerEvent::Disconnected => self.on_game_server_disconnected(),
            },
            ServerEvent::Room(room_event) => match room_event {
                RoomEvent::PresenceEnter(name) => self.on_player_entered(name),
                RoomEvent::PresenceLeave(name) => self.on_player_left(name),
                RoomEvent::Spawn(spawn_data) => match spawn_data.r#type.as_str() {
                    "NPC" => self.on_npc_spawned(spawn_data),
                    "ITEM" => self.on_item_spawned(spawn_data),
                    t => {
                        self.state.ui.notifications.push(
                            Notification::warning(format!("Unknown spawn event: {}", t))
                                .with_topic(NotificationTopic::Protocol),
                        );
                    }
                },
                RoomEvent::Despawn(spawn_data) => match spawn_data.r#type.as_str() {
                    "ITEM" => self.on_item_despawned(spawn_data),
                    t => {
                        self.state.ui.notifications.push(
                            Notification::warning(format!("Unknown despawn event: {}", t))
                                .with_topic(NotificationTopic::Protocol),
                        );
                    }
                },
                RoomEvent::Take(player, item_id) => self.on_item_taken_by(player, item_id),
                RoomEvent::Drop(player, item_id) => self.on_item_dropped_by(player, item_id),
            },
            ServerEvent::Group(group_event) => match group_event {
                GroupEvent::Invite(leader) => self.on_group_invited_by(leader),
                GroupEvent::InviteRemoved(leader) => self.on_group_invite_removed(leader),
                GroupEvent::Join(user) => self.on_group_member_joined(user),
                GroupEvent::Leave(user) => self.on_group_member_left(user),
                GroupEvent::Move(direction) => self.on_group_moved(direction),
            },
            ServerEvent::Chat(chat_event) => match chat_event {
                ChatEvent::Global(chat) => {
                    self.on_chat_received(ChatChannel::Global, chat.sender, chat.message)
                }
                ChatEvent::Room(chat) => {
                    self.on_chat_received(ChatChannel::Room, chat.sender, chat.message)
                }
                ChatEvent::Group(chat) => {
                    self.on_chat_received(ChatChannel::Group, chat.sender, chat.message)
                }
                ChatEvent::Private(chat) => {
                    let channel = ChatChannel::Private(chat.sender.clone());
                    self.on_chat_received(channel, chat.sender, chat.message);
                }
            },
            ServerEvent::Fight(fight_event) => match fight_event {
                FightEvent::Start(data) => self.on_fight_start(data),
                FightEvent::Result(data) => self.on_fight_result(data),
                FightEvent::End(data) => self.on_fight_end(data),
            },
            ServerEvent::Quest(quest_event) => match quest_event {
                QuestEvent::Add(data) => self.on_quest_add(data),
                QuestEvent::Step(data) => self.on_quest_step(data),
                QuestEvent::Complete(data) => self.on_quest_complete(data),
            },
            ServerEvent::Item(item_event) => match item_event {
                ItemEvent::Add(item_identifier) => self.on_item_add(item_identifier),
            },
            ServerEvent::Kill(kill_data) => self.on_kill(kill_data),
            ServerEvent::Death(death_data) => self.on_death(death_data),
            ServerEvent::CounterAttack(counter_attack) => self.on_counter_attack(counter_attack),
            ServerEvent::Teleport => self.on_teleport(),
            ServerEvent::Unknown(raw) => self.on_unknown_event(raw),
        }
    }
}
