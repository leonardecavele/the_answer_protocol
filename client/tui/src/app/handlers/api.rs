use crate::app::App;
use crate::events::ApiEvent;
use crate::network::envelopes::ResponseEnvelope;
use crate::states::game::{ChatChannel, ChatMessage, DialogueState, END_OF_DIALOGUE_TAG};
use crate::states::ui::Notification;
use api_client::client::event::{GameServerEvent, GroupEvent, RoomEvent, ServerEvent};
use api_client::protocol::command::core::look::LookCommand;
use api_client::protocol::command::enums::{ApiRequest, ApiResponse};
use api_client::protocol::command::resource_interaction::quests::QuestListEntry;

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
                self.push_event(
                    "api request",
                    format!("{} - {:?}", envelope.id, envelope.request),
                );
            }
        }
    }

    pub(crate) fn handle_api_response(&mut self, envelope: ResponseEnvelope) {
        if let Some(error) = envelope.response.get_error() {
            self.push_event(
                "api response",
                format!("[ERROR] {} - {}", envelope.id, error),
            );
            self.state
                .ui
                .notification
                .push(Notification::warning(error.to_string()));

            return;
        }

        self.push_event(
            "api response",
            format!("{} - {:?}", envelope.id, envelope.response),
        );

        self.state.network.is_connected = true;

        match envelope.response {
            ApiResponse::Connect(Ok(connect_res)) => {
                self.state.game.player.name = Some(connect_res.player_name);
            }
            ApiResponse::Who(Ok(who_res)) => {
                self.state.game.server.online_players_count = who_res.player_count;
                self.state
                    .game
                    .log_action("You checked who is here.".to_string());
            }
            ApiResponse::Status(Ok(status_res)) => {
                self.state.game.player.hp = status_res.player_status.hp;
                self.state.game.player.max_hp = status_res.player_status.max_hp;
                self.state
                    .game
                    .log_action("You checked your state.".to_string());
            }
            ApiResponse::GroupCreate(Ok(res)) => {
                self.state.game.group.id = Some(res.group_id.clone());
                self.state.game.group.leader = self.state.game.player.name.clone();
                self.state
                    .game
                    .log_action(format!("You created group {}.", res.group_id));
            }
            ApiResponse::GroupJoin(Ok(res)) => {
                if let ApiRequest::GroupJoin(cmd) = envelope.original_request {
                    self.state.game.group.id = Some(res.group_id);
                    self.state.game.group.leader = Some(cmd.leader_name.to_uppercase());
                    self.state
                        .game
                        .log_action(format!("You joined the group of {}.", cmd.leader_name));
                }
            }
            ApiResponse::GroupLeave(Ok(_res)) => {
                self.state.game.group.id = None;
                self.state.game.group.leader = None;
                self.state
                    .game
                    .log_action("You left the group.".to_string());
            }
            ApiResponse::GlobalChat(Ok(_)) => {
                if let ApiRequest::GlobalChat(cmd) = envelope.original_request {
                    self.state.game.chat_history.push(ChatMessage {
                        channel: ChatChannel::Global,
                        sender: "You".to_string(),
                        content: cmd.message,
                    });
                }
            }
            ApiResponse::PrivateChat(Ok(_)) => {
                if let ApiRequest::PrivateChat(cmd) = envelope.original_request {
                    self.state.game.chat_history.push(ChatMessage {
                        channel: ChatChannel::Private(cmd.to.clone()),
                        sender: format!("(You) to {}", cmd.to),
                        content: cmd.message,
                    });
                }
            }
            ApiResponse::Look(Ok(mut look_res)) => {
                self.state
                    .game
                    .log_action(format!("You looked at {}.", look_res.room.name));

                let player_name = self.state.game.player.name.as_ref();
                look_res.players.retain(|p| {
                    !p.eq_ignore_ascii_case(player_name.unwrap_or(&"unknown".to_string()).as_str())
                });

                self.state.game.room.id = Some(look_res.room.id);
                self.state.game.room.name = Some(look_res.room.name);
                self.state.game.room.description = Some(look_res.room.description);
                self.state.game.room.players = look_res.players;
                self.state.game.room.npcs = look_res.npcs;
                self.state.game.room.items = look_res.items;
                self.state.game.room.exits = look_res.room.exits;
            }
            ApiResponse::Move(Ok(_move_res)) => {
                self.state.game.ui.focused_entity_id = None;
                if let ApiRequest::Move(cmd) = envelope.original_request {
                    self.state
                        .game
                        .log_action(format!("You moved {}.", cmd.direction));
                }
                self.handle_request(ApiRequest::Look(LookCommand));
            }
            ApiResponse::Inventory(Ok(inv_res)) => {
                self.state.game.player.inventory = inv_res.inventory;
                self.state
                    .game
                    .log_action("You checked your inventory.".to_string());
            }
            ApiResponse::Quests(Ok(quests_res)) => {
                self.state.game.player.quests = quests_res.quest_list;
                self.state
                    .game
                    .log_action("You checked your quests.".to_string());
            }
            ApiResponse::Quest(Ok(quest_res)) => {
                self.state.game.player.quests.push(QuestListEntry {
                    quest_id: quest_res.quest_data.quest_id,
                    status: quest_res.quest_data.status,
                    progress: Some("in progress".to_string()),
                });
            }
            ApiResponse::Talk(Ok(talk_res)) => {
                if let ApiRequest::Talk(cmd) = envelope.original_request {
                    let mut text = talk_res.dialogue;
                    let ends_dialog = text.contains(END_OF_DIALOGUE_TAG);
                    let ends_dialog_only = ends_dialog && text.starts_with(END_OF_DIALOGUE_TAG);

                    if ends_dialog_only {
                        if self.state.game.ui.active_dialogue.is_none() {
                            text = "**nothing**".to_string();
                        } else {
                            self.state.game.ui.close_dialogue();
                            return;
                        }
                    } else if ends_dialog {
                        text = text.replace(END_OF_DIALOGUE_TAG, "").trim().to_string();
                    }

                    self.state.game.ui.focused_entity_id = Some(cmd.npc_name.clone());

                    let display_name = self.state.game.manifest.get_npc_name(&cmd.npc_name);

                    if self.state.game.ui.active_dialogue.is_none() {
                        self.state
                            .game
                            .log_action(format!("You talked to {}.", display_name));
                    }

                    if !ends_dialog_only {
                        self.state
                            .game
                            .log_action(format!("[{}] says: \"{}\"", display_name, text));
                    }

                    if let Some(ref mut dialog) = self.state.game.ui.active_dialogue {
                        dialog.add(text, ends_dialog);
                    } else {
                        self.state.game.ui.active_dialogue = Some(DialogueState::new(
                            cmd.npc_name,
                            display_name,
                            text,
                            ends_dialog,
                        ));
                    }
                }
            }
            ApiResponse::Take(Ok(take_res)) => {
                self.state
                    .game
                    .log_action(format!("You took {}.", take_res.item_identifier));
                self.state
                    .game
                    .room
                    .items
                    .retain(|i| i != &take_res.item_identifier);
                self.state
                    .game
                    .player
                    .inventory
                    .push(take_res.item_identifier);
            }
            ApiResponse::Drop(Ok(drop_res)) => {
                self.state
                    .game
                    .log_action(format!("You dropped {}.", drop_res.item_identifier));
                self.state
                    .game
                    .player
                    .inventory
                    .retain(|item| !item.eq(&drop_res.item_identifier));
                self.state.game.room.items.push(drop_res.item_identifier);
            }
            ApiResponse::Attack(Ok(attack_res)) => {
                if let ApiRequest::Attack(cmd) = envelope.original_request {
                    self.state.game.ui.focused_entity_id = Some(cmd.npc_name.clone());

                    let display_name = self.state.game.manifest.get_npc_name(&cmd.npc_name);

                    self.state
                        .game
                        .log_action(format!("You attacked {}.", display_name));

                    let res = attack_res.combat_result;

                    // Update HP manually from attack result
                    self.state.game.player.hp = res.attacker_hp;

                    let text = match res.status.eq_ignore_ascii_case("Victory") {
                        true => {
                            self.state.game.room.npcs.retain(|npc| npc != &cmd.npc_name);
                            format!(
                                "Combat with {}: You dealt {} damage. {} is death. Victory.",
                                display_name, res.damage, display_name
                            )
                        }
                        false => {
                            format!(
                                "Combat with {}: You dealt {} damage. (Your HP: {} | Target HP: {}) ",
                                display_name, res.damage, res.attacker_hp, res.target_hp
                            )
                        }
                    };

                    self.state.game.ui.active_dialogue = Some(DialogueState::new(
                        cmd.npc_name,
                        display_name.clone(),
                        text.clone(),
                        true,
                    ));

                    self.state.game.log_action(text);
                }
            }
            ApiResponse::Quit(Ok(_)) => {
                self.state.should_quit = true;
            }
            response => {
                self.state.ui.event_history.insert(
                    0,
                    format!(
                        "Missing handle response for command: {:?} -- Response: {:?}",
                        envelope.original_request, response
                    ),
                );
            }
        }
    }

    pub(crate) fn handle_server_event(&mut self, event: ServerEvent) {
        self.push_event("server", format!("{:?}", event));

        match event {
            ServerEvent::Connect(name) => {
                self.state.game.server.online_players_count += 1;
                self.state
                    .game
                    .log_action(format!("{} joined the server.", name));
            }
            ServerEvent::Spawn(spawn_data) => match spawn_data.r#type.as_str() {
                "NPC" => {
                    let npc_name = self
                        .state
                        .game
                        .manifest
                        .get_npc_name(spawn_data.id.as_str());

                    self.state
                        .game
                        .log_action(format!("{} has respawn", npc_name));
                    self.state.game.room.npcs.push(spawn_data.id);
                }
                "ITEM" => {
                    let item_name = self
                        .state
                        .game
                        .manifest
                        .get_item_name(spawn_data.id.as_str());
                    self.state
                        .game
                        .log_action(format!("{} has been catapulted here", item_name));
                    self.state.game.room.items.push(spawn_data.id);
                }
                t => {
                    self.state
                        .ui
                        .notification
                        .push(Notification::warning(format!("Unknown spawn event: {}", t)));
                }
            },
            ServerEvent::Despawn(spawn_data) => match spawn_data.r#type.as_str() {
                "NPC" => {
                    let npc_name = self
                        .state
                        .game
                        .manifest
                        .get_npc_name(spawn_data.id.as_str());

                    self.state
                        .game
                        .log_action(format!("{} has been defeated", npc_name));
                    self.state
                        .game
                        .room
                        .npcs
                        .retain(|npc| npc != spawn_data.id.as_str());
                }
                "ITEM" => {
                    let item_name = self
                        .state
                        .game
                        .manifest
                        .get_item_name(spawn_data.id.as_str());
                    self.state
                        .game
                        .log_action(format!("{} has despawned", item_name));
                    self.state
                        .game
                        .room
                        .items
                        .retain(|item| item != spawn_data.id.as_str());
                }
                t => {
                    self.state
                        .ui
                        .notification
                        .push(Notification::warning(format!(
                            "Unknown despawn event: {}",
                            t
                        )));
                }
            },
            ServerEvent::Quit(name) => {
                self.state.game.server.online_players_count = self
                    .state
                    .game
                    .server
                    .online_players_count
                    .saturating_sub(1);
                self.state.game.room.players.retain(|p| p != &name);
                if self.state.game.group.leader.as_ref() == Some(&name) {
                    self.state.game.group.id = None;
                    self.state.game.group.leader = None;
                }
                self.state
                    .game
                    .log_action(format!("{} disconnected.", name));
            }
            ServerEvent::Room(room_event) => match room_event {
                RoomEvent::PresenceEnter(name) => {
                    if !self.state.game.room.players.contains(&name) {
                        self.state.game.room.players.push(name.clone());
                    }

                    self.state
                        .game
                        .log_action(format!("{} entered the room.", name));
                }
                RoomEvent::PresenceLeave(name) => {
                    self.state.game.room.players.retain(|p| p != &name);
                    self.state
                        .game
                        .log_action(format!("{} left the room.", name));
                }
                RoomEvent::Chat(chat) => {
                    self.state.game.chat_history.push(ChatMessage {
                        channel: ChatChannel::Room,
                        sender: chat.sender,
                        content: chat.message,
                    });
                }
                RoomEvent::Take(player, item) => {
                    self.state.game.room.items.retain(|id| id != &item);
                    self.state
                        .game
                        .log_action(format!("{} took {}.", player, item));
                }
                RoomEvent::Drop(player, item) => {
                    self.state.game.room.items.push(item.clone());
                    self.state
                        .game
                        .log_action(format!("{} dropped {}.", player, item));
                }
            },
            ServerEvent::Group(group_event) => match group_event {
                GroupEvent::Invite(leader) => {
                    self.state.ui.notification.push(Notification::info(format!(
                        "You are invited to a group by {}.",
                        leader
                    )));
                }
                GroupEvent::Join(user) => {
                    self.state
                        .game
                        .log_action(format!("{} joined the group.", user));
                }
                GroupEvent::Leave(user) => {
                    if self.state.game.group.leader.as_ref() == Some(&user.to_uppercase()) {
                        self.state.game.group.id = None;
                        self.state.game.group.leader = None;

                        self.state.game.log_action(format!(
                            "Leader {} left. The group has been disbanded.",
                            user
                        ));
                    } else {
                        self.state
                            .game
                            .log_action(format!("{} left the group.", user));
                    }
                }
                GroupEvent::Chat(chat) => {
                    self.state.game.chat_history.push(ChatMessage {
                        channel: ChatChannel::Group,
                        sender: chat.sender,
                        content: chat.message,
                    });
                }
                GroupEvent::Move(direction) => {
                    self.state
                        .game
                        .log_action(format!("Group moved to {}.", direction));

                    self.handle_request(ApiRequest::Look(LookCommand));
                }
            },
            ServerEvent::GlobalChat(chat) => {
                self.state.game.chat_history.push(ChatMessage {
                    channel: ChatChannel::Global,
                    sender: chat.sender,
                    content: chat.message,
                });
            }
            ServerEvent::PrivateChat(chat) => {
                self.state.game.chat_history.push(ChatMessage {
                    channel: ChatChannel::Private(chat.sender.clone()),
                    sender: chat.sender.clone(),
                    content: chat.message,
                });
                self.state.ui.notification.push(Notification::info(format!(
                    "New private message from {}.",
                    chat.sender
                )));
            }
            ServerEvent::Stats(count) => {
                self.state.game.server.online_players_count = count;
            }
            ServerEvent::Unknown(raw) => {
                self.state
                    .ui
                    .notification
                    .push(Notification::warning(format!("Unknown event: {}", raw)));
            }
            ServerEvent::GameServer(game_server_event) => match game_server_event {
                GameServerEvent::Connected => {
                    self.state
                        .game
                        .log_action("Game server online.".to_string());

                    self.state.ui.notification.push(Notification::info(
                        "Game server is online. Session restarted.",
                    ));

                    self.handle_request(ApiRequest::Look(LookCommand));
                    self.load_state_from_server();
                }
                GameServerEvent::Disconnected => {
                    self.state
                        .game
                        .log_action("Game server offline.".to_string());

                    self.state.network.is_connected = false;
                }
            },
            _ => {
                self.state
                    .ui
                    .notification
                    .push(Notification::warning(format!(
                        "Missing handler for event: {:?}",
                        event
                    )));
            }
        }
    }
}
