use crate::app::App;
use crate::network::envelopes::ResponseEnvelope;
use api_client::protocol::command::enums::ApiResponse;

impl App {
    pub fn handle_api_response(&mut self, envelope: ResponseEnvelope) {
        match envelope.response {
            ApiResponse::Connect(Ok(connect_res)) => {
                self.state.game.player_name = Some(connect_res.player_name);
            }
            ApiResponse::Who(Ok(who_res)) => {
                self.state.game.online_players_count = who_res.player_count;
            }
            ApiResponse::Status(Ok(status_res)) => {
                self.state.game.hp = status_res.player_status.hp;
                self.state.game.max_hp = status_res.player_status.max_hp;
            }
            ApiResponse::GroupCreate(Ok(res)) => {
                self.state.game.group_id = Some(res.group_id.clone());
                self.state.game.group_leader = self.state.game.player_name.clone();
            }
            ApiResponse::GroupJoin(Ok(res)) => {
                if let api_client::protocol::command::enums::ApiRequest::GroupJoin(cmd) =
                    envelope.original_request
                {
                    self.state.game.group_id = Some(res.group_id.clone());
                    self.state.game.group_leader = Some(cmd.leader_name.to_uppercase());
                }
            }
            ApiResponse::GroupLeave(Ok(_res)) => {
                self.state.game.group_id = None;
                self.state.game.group_leader = None;
            }
            ApiResponse::GlobalChat(Ok(_)) => {
                if let api_client::protocol::command::enums::ApiRequest::GlobalChat(cmd) =
                    envelope.original_request
                {
                    self.state
                        .game
                        .chat_history
                        .push(crate::states::game::ChatMessage {
                            channel: crate::states::game::ChatChannel::Global,
                            sender: "You".to_string(),
                            content: cmd.message.clone(),
                        });
                }
            }
            ApiResponse::PrivateChat(Ok(_)) => {
                if let api_client::protocol::command::enums::ApiRequest::PrivateChat(cmd) =
                    envelope.original_request
                {
                    self.state
                        .game
                        .chat_history
                        .push(crate::states::game::ChatMessage {
                            channel: crate::states::game::ChatChannel::Private(cmd.to.clone()),
                            sender: format!("(You) to {}", cmd.to),
                            content: cmd.message.clone(),
                        });
                }
            }
            ApiResponse::Look(Ok(look_res)) => {
                self.state.game.current_room_id = Some(look_res.room.id.clone());
                self.state.game.current_room_name = Some(look_res.room.name.clone());
                self.state.game.current_room_description = Some(look_res.room.description.clone());
                self.state.game.room_players = look_res.players.clone();
                self.state.game.room_npcs = look_res.npcs.clone();
                self.state.game.current_room_items = look_res.items.clone();
                self.state.game.current_room_exits = look_res.room.exits.clone();
            }
            ApiResponse::Move(Ok(_move_res)) => {
                self.state.game.focused_entity_id = None;
                if let Some(network_manager) = &self.network_manager {
                    let req = api_client::protocol::command::enums::ApiRequest::Look(
                        api_client::protocol::command::core::look::LookCommand,
                    );
                    let envelope = crate::network::envelopes::RequestEnvelope::new(req);
                    network_manager.send_command(envelope);
                }
            }
            ApiResponse::Inventory(Ok(inv_res)) => {
                self.state.game.inventory = inv_res.inventory.clone();
            }
            ApiResponse::Quests(Ok(quests_res)) => {
                self.state.game.quests = quests_res.quest_list.clone();
            }
            ApiResponse::Talk(Ok(talk_res)) => {
                if let api_client::protocol::command::enums::ApiRequest::Talk(cmd) =
                    envelope.original_request
                {
                    let mut text = talk_res.dialogue.clone();
                    let ends_dialog = text.contains(crate::states::game::END_OF_DIALOGUE_TAG);
                    if ends_dialog && text.starts_with(crate::states::game::END_OF_DIALOGUE_TAG) {
                        text = text
                            .replace(crate::states::game::END_OF_DIALOGUE_TAG, "**nothing**")
                            .trim()
                            .to_string();
                    } else if ends_dialog {
                        text = text
                            .replace(crate::states::game::END_OF_DIALOGUE_TAG, "")
                            .trim()
                            .to_string();
                    }

                    self.state.game.focused_entity_id = Some(cmd.npc_name.clone());

                    let display_name = self.state.game.manifest.get_npc_name(&cmd.npc_name);

                    self.state
                        .game
                        .log_action(format!("[{}] says: \"{}\"", display_name, text));

                    self.state.game.active_dialogue =
                        Some(crate::states::game::DialogueState::new(
                            cmd.npc_name,
                            display_name,
                            text,
                            ends_dialog,
                        ));
                }
            }
            ApiResponse::Take(Ok(take_res)) => {
                self.state
                    .game
                    .inventory
                    .push(take_res.item_identifier.clone());
                self.state
                    .game
                    .current_room_items
                    .retain(|i| i != &take_res.item_identifier);
            }
            ApiResponse::Drop(Ok(drop_res)) => {
                self.state
                    .game
                    .current_room_items
                    .push(drop_res.item_identifier.clone());
                self.state
                    .game
                    .inventory
                    .retain(|item| !item.eq(&drop_res.item_identifier));
            }
            ApiResponse::Attack(Ok(attack_res)) => {
                if let api_client::protocol::command::enums::ApiRequest::Attack(cmd) =
                    envelope.original_request
                {
                    self.state.game.focused_entity_id = Some(cmd.npc_name.clone());

                    let display_name = self.state.game.manifest.get_npc_name(&cmd.npc_name);

                    let res = attack_res.combat_result;

                    // Update HP manually from attack result
                    self.state.game.hp = res.attacker_hp;

                    let text = match res.status.eq_ignore_ascii_case("Victory") {
                        true => {
                            self.state.game.room_npcs.retain(|npc| npc != &cmd.npc_name);
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

                    self.state.game.active_dialogue =
                        Some(crate::states::game::DialogueState::new(
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
            // Add other successful response handlers here as needed
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
}
