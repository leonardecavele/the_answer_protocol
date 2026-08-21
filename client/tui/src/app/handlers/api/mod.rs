mod chat;
mod combat;
mod group;
mod player;
mod room;
mod server;

use crate::app::App;
use crate::events::ApiEvent;
use crate::network::envelopes::ResponseEnvelope;
use crate::states::game::{ChatChannel, DialogueState, END_OF_DIALOGUE_TAG, Overlay, OverlayKind};
use crate::states::ui::Notification;
use api_client::events::{GameServerEvent, GroupEvent, RoomEvent, ServerEvent};
use api_client::{ApiRequest, ApiResponse};

impl App {
    pub(crate) fn handle_api_event(&mut self, api_event: ApiEvent) {
        match api_event {
            ApiEvent::Server(server_event) => {
                self.handle_server_event(server_event);
            }
            ApiEvent::ApiResponse(envelope) => {
                self.handle_api_response(envelope);
            }
            ApiEvent::LogApiRequest(envelope) => {
                self.record_trace(
                    "api request",
                    format!("{} - {:?}", envelope.id, envelope.request),
                );
            }
        }
    }

    pub(crate) fn handle_api_response(&mut self, envelope: ResponseEnvelope) {
        if let Some(error) = envelope.response.get_error() {
            self.record_trace(
                "api response",
                format!("[ERROR] {} - {}", envelope.id, error),
            );
            self.state
                .ui
                .notifications
                .push(Notification::warning(error.to_string()));

            return;
        }

        self.record_trace(
            "api response",
            format!("{} - {:?}", envelope.id, envelope.response),
        );

        match envelope.response {
            ApiResponse::Connect(Ok(response)) => {
                self.on_connected(response);
            }
            ApiResponse::Who(Ok(response)) => {
                self.on_who(response);
            }
            ApiResponse::FightCreate(Ok(_)) => {}
            ApiResponse::FightAttack(Ok(_)) => {}
            ApiResponse::Status(Ok(response)) => {
                self.on_status(response);
            }
            ApiResponse::GroupCreate(Ok(response)) => {
                self.on_group_created(response);
            }
            ApiResponse::GroupJoin(Ok(response)) => {
                if let ApiRequest::GroupJoin(cmd) = envelope.original_request {
                    self.on_group_joined(response, cmd.leader_name);
                }
            }
            ApiResponse::GroupLeave(Ok(_)) => {
                self.on_group_left();
            }
            ApiResponse::GlobalChat(Ok(_)) => {
                if let ApiRequest::GlobalChat(cmd) = envelope.original_request {
                    self.on_global_chat_sent(cmd.message);
                }
            }
            ApiResponse::PrivateChat(Ok(_)) => {
                if let ApiRequest::PrivateChat(cmd) = envelope.original_request {
                    self.on_private_chat_sent(cmd.to, cmd.message);
                }
            }
            ApiResponse::Look(Ok(response)) => {
                self.on_look(response);
            }
            ApiResponse::Move(Ok(_)) => {
                if let ApiRequest::Move(cmd) = envelope.original_request {
                    self.on_moved(cmd.direction);
                }
            }
            ApiResponse::Inventory(Ok(response)) => {
                self.on_inventory(response);
            }
            ApiResponse::Quests(Ok(response)) => {
                self.on_quests(response);
            }
            ApiResponse::Quest(Ok(response)) => {
                self.on_quest(response);
            }
            ApiResponse::Talk(Ok(response)) => {
                if let ApiRequest::Talk(cmd) = envelope.original_request {
                    let mut text = response.dialogue;
                    let ends_dialog = text.contains(END_OF_DIALOGUE_TAG);
                    let ends_dialog_only = ends_dialog && text.starts_with(END_OF_DIALOGUE_TAG);

                    if ends_dialog_only {
                        if !self.state.game.overlays.is_open(OverlayKind::Dialogue) {
                            text = "**nothing**".to_string();
                        } else {
                            self.state.game.overlays.close(OverlayKind::Dialogue);
                            return;
                        }
                    } else if ends_dialog {
                        text = text.replace(END_OF_DIALOGUE_TAG, "").trim().to_string();
                    }

                    self.state.game.overlays.inspected_entity = Some(cmd.npc_name.clone());

                    let display_name = self.state.game.manifest.npc_name(&cmd.npc_name);

                    if !self.state.game.overlays.is_open(OverlayKind::Dialogue) {
                        self.state
                            .game
                            .log_action(format!("You talked to {}.", display_name));
                    }

                    if !ends_dialog_only {
                        self.state
                            .game
                            .log_action(format!("[{}] says: \"{}\"", display_name, text));
                    }

                    if let Some(dialog) = self.state.game.overlays.dialogue_mut() {
                        dialog.add(text, ends_dialog);
                    } else {
                        self.state
                            .game
                            .overlays
                            .open(Overlay::Dialogue(DialogueState::new(
                                cmd.npc_name,
                                display_name,
                                text,
                                ends_dialog,
                            )));
                    }
                }
            }
            ApiResponse::Take(Ok(response)) => {
                self.on_take_item(response);
            }
            ApiResponse::Drop(Ok(response)) => {
                self.on_drop_item(response);
            }
            ApiResponse::Attack(Ok(response)) => {
                if let ApiRequest::Attack(cmd) = envelope.original_request {
                    self.state.game.overlays.inspected_entity = Some(cmd.npc_name.clone());

                    let npc_name = self.state.game.manifest.npc_name(&cmd.npc_name);

                    self.state
                        .game
                        .log_action(format!("You attacked {}.", npc_name));

                    let res = response.combat_result;

                    // Update HP manually from attack result
                    self.state.game.player.hp = res.attacker_hp;

                    let text = format!(
                        "Combat with {}: You dealt {} damage. (Your HP: {} | Target HP: {}) ",
                        npc_name, res.damage, res.attacker_hp, res.target_hp
                    );

                    self.state
                        .game
                        .overlays
                        .open(Overlay::Dialogue(DialogueState::new(
                            cmd.npc_name,
                            npc_name,
                            text.clone(),
                            true,
                        )));

                    self.state.game.log_action(text);
                }
            }
            ApiResponse::Quit(Ok(_)) => {
                self.state.should_quit = true;
            }
            response => {
                self.state.ui.trace_log.push(format!(
                    "Missing handle response for command: {:?} -- Response: {:?}",
                    envelope.original_request, response
                ));
            }
        }
    }

    pub(crate) fn handle_server_event(&mut self, event: ServerEvent) {
        self.record_trace("server", format!("{:?}", event));

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
