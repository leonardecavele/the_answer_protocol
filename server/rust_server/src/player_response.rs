use crate::{game_manager::GameManager};
use json::{object};
use tracing::debug;
impl GameManager {
        pub fn send_diff_to_players(&mut self) -> std::io::Result<()> {
        let diff = self.get_tick_diff().clone();

        if diff.is_empty() {
            return Ok(());
        }

        for (player_name, tick_diff) in diff.iter() {
            let msg = object! {
                "player": player_name.as_str(),
                "events": tick_diff.clone()
            };
            debug!("sent event to {}: {}", player_name,  msg);
            self.send_msg_to_client(msg.dump())?;
        }

        Ok(())
    }
}