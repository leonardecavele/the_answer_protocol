use crate::app::App;
use crate::network::envelopes::ResponseEnvelope;
use api_client::protocol::command::enums::ApiResponse;

impl App {
    pub fn handle_api_response(&mut self, envelope: ResponseEnvelope) {
        match envelope.response {
            ApiResponse::Look(Ok(look_res)) => {
                self.state.game.current_room_id = Some(look_res.room.id.clone());
                self.state.game.room_players = look_res.players.clone();
                self.state.game.room_npcs = look_res.npcs.clone();
            }
            ApiResponse::Move(Ok(_move_res)) => {
                // We successfully moved. Wait for ServerEvent::Room to update UI,
                // or update UI proactively if MoveResponse has data.
            }
            ApiResponse::Inventory(Ok(_inv_res)) => {
                // Update inventory
            }
            // Add other successful response handlers here as needed
            _ => {}
        }
    }
}
