use crate::app::App;
use crate::network::envelopes::ResponseEnvelope;
use api_client::protocol::command::enums::ApiResponse;

impl App {
    pub fn handle_api_response(&mut self, envelope: ResponseEnvelope) {
        match envelope.response {
            ApiResponse::Look(Ok(look_res)) => {
                self.state.game.current_room_id = Some(look_res.room.id.clone());
                self.state.game.current_room_name = Some(look_res.room.name.clone());
                self.state.game.current_room_description = Some(look_res.room.description.clone());
                self.state.game.room_players = look_res.players.clone();
                self.state.game.room_npcs = look_res.npcs.clone();
                self.state.game.current_room_exits = look_res.room.exits.clone();
            }
            ApiResponse::Move(Ok(_move_res)) => {
                self.state.game.focused_entity_id = None;
                if let Some(network_manager) = &self.network_manager {
                    let req = api_client::protocol::command::enums::ApiRequest::Look(api_client::protocol::command::core::look::LookCommand);
                    let envelope = crate::network::envelopes::RequestEnvelope::new(req);
                    network_manager.send_command(envelope);
                }
            }
            ApiResponse::Inventory(Ok(_inv_res)) => {
                // Update inventory
            }
            ApiResponse::Talk(Ok(talk_res)) => {
                if let api_client::protocol::command::enums::ApiRequest::Talk(cmd) = envelope.original_request {
                    let mut text = talk_res.dialogue.clone();
                    let ends_dialog = text.contains(crate::states::game::END_OF_DIALOGUE_TAG);
                    if ends_dialog {
                        text = text.replace(crate::states::game::END_OF_DIALOGUE_TAG, "**nothing**").trim().to_string();
                    }

                    self.state.game.focused_entity_id = Some(cmd.npc_name.clone());
                    
                    let display_name = self.state.game.manifest.npcs.get(&cmd.npc_name)
                        .map(|n| n.name.clone())
                        .unwrap_or_else(|| cmd.npc_name.clone());
                    
                    self.state.game.log_action(format!("[{}] says: \"{}\"", display_name, text));
                    
                    self.state.game.active_dialogue = Some(crate::states::game::DialogueState::new(
                        cmd.npc_name,
                        display_name,
                        text,
                        ends_dialog,
                    ));
                }
            }
            ApiResponse::Attack(Ok(attack_res)) => {
                if let api_client::protocol::command::enums::ApiRequest::Attack(cmd) = envelope.original_request {
                    self.state.game.focused_entity_id = Some(cmd.npc_name.clone());
                    
                    let display_name = self.state.game.manifest.npcs.get(&cmd.npc_name)
                        .map(|n| n.name.clone())
                        .unwrap_or_else(|| cmd.npc_name.clone());

                    let res = attack_res.combat_result;
                    self.state.game.log_action(format!(
                        "Combat with {}: You dealt {} damage. (Your HP: {} | Target HP: {}) Status: {}",
                        display_name, res.damage, res.attacker_hp, res.target_hp, res.status
                    ));
                }
            }
            // Add other successful response handlers here as needed
            _ => {}
        }
    }
}
