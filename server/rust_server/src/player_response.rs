use crate::constants::TickResult;
use crate::game_manager::GameManager;
use json::object;
use tracing::debug;

impl GameManager {
    pub fn send_diff_to_players(&mut self) -> TickResult {
        let diff = self.get_tick_diff().clone();

        if diff.is_empty() {
            return TickResult::TickEnd;
        }

        for (player_name, tick_diff) in diff.iter() {
            let msg = object! {
                "player": player_name.as_str(),
                "events": tick_diff.clone()
            };
            debug!("sent event to {}: {}", player_name, msg);
            if self.send_msg_to_client(msg.dump()) == TickResult::Exit {
                return TickResult::Exit;
            }
        }

        TickResult::TickEnd
    }
}
