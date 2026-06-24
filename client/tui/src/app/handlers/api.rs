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
                // We successfully moved. Wait for ServerEvent::Room to update UI,
                // or update UI proactively if MoveResponse has data.
            }
            ApiResponse::Inventory(Ok(_inv_res)) => {
                // Update inventory
            }
            ApiResponse::Talk(Ok(talk_res)) => {
                if let api_client::protocol::command::enums::ApiRequest::Talk(cmd) = envelope.original_request {
                    self.state.game.focused_entity_id = Some(cmd.npc_name.clone());
                    self.state.game.log_action(format!("[{}] says: \"{}\"", cmd.npc_name, talk_res.dialogue));
                }
            }
            ApiResponse::Attack(Ok(attack_res)) => {
                if let api_client::protocol::command::enums::ApiRequest::Attack(cmd) = envelope.original_request {
                    self.state.game.focused_entity_id = Some(cmd.npc_name.clone());
                    let res = attack_res.combat_result;
                    self.state.game.log_action(format!(
                        "Combat with {}: You dealt {} damage. (Your HP: {} | Target HP: {}) Status: {}",
                        cmd.npc_name, res.damage, res.attacker_hp, res.target_hp, res.status
                    ));
                }
            }
            // Add other successful response handlers here as needed
            _ => {}
        }
    }
}
