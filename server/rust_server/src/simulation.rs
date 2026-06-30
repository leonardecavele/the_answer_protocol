use crate::constantes::{TICK_TIME, TickResult};
use crate::game_manager::GameManager;
use std::sync::mpsc;
use std::time::Instant;

impl GameManager {
    pub fn apply_players_changes(&mut self, tick_timer: Instant) -> std::io::Result<TickResult> {
        loop {
            if tick_timer.elapsed() >= TICK_TIME {
                break;
            }
            match self.receive_data_timeout(TICK_TIME - tick_timer.elapsed()) {
                Ok(msg) => {
                    let command_response = self.handle_message(msg);
                    self.send_msg_to_client(command_response)?;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(TickResult::Exit),
            };

        }
        return Ok(TickResult::TickEnd);
    }

    pub fn update_game_state(&mut self) -> std::io::Result<()> {
        for quest_instance in self.quest_instances.iter_mut() {
            let state = quest_instance.get_state();
            match quest_instance.get_quest_name().as_str() {
                "Tunnel" => {
                    
                }
                _ => {}
                // => {}
                // => {}
            }
        }
        Ok(())
    }
}
