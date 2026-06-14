use crate::constantes::{TICK_TIME, TickResult};
use crate::game_manager::GameManager;
use json::{JsonValue, object};
use std::sync::mpsc;
use std::time::Instant;
use tracing::info;

impl GameManager {
    pub fn apply_players_changes(&mut self, tick_timer: Instant) -> std::io::Result<TickResult> {
        loop {
            if tick_timer.elapsed() >= TICK_TIME {
                break;
            }
            match self.receive_data_timeout(TICK_TIME - tick_timer.elapsed())
            {
                Ok(msg) => {
                    let command_response = self.handle_message(msg);
                    self.send_msg_to_client(command_response + "\n")?;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(TickResult::Exit),
            };
        }
        return Ok(TickResult::TickEnd);
    }

    pub fn update_game_state(&mut self) -> std::io::Result<()> {
        /*
        here we should update the game state ( npcs, monsters, mouvements, etc
        and store the diff in a buffer (an argument of this function),
        we will send it back to players at the end of the tick
        */

        //hardcoded event for now :

        let event_type = "partouze";
        let mut all_events: Vec<JsonValue> = vec![];
        for (player_name, _) in self.get_players_by_names() {
            if player_name == "GABIN" || player_name == "AYMERIC" {
                all_events.push(object! {
                    "player": player_name.as_str(),
                    "event_name": event_type,
                    "value": ""
                });
            }
        }
        let array = JsonValue::Array(all_events);
        if array.is_empty() {
            return Ok(());
        }
        self.send_msg_to_client(array.dump() + "\n")?;

        info!("sent event");
        Ok(())
    }
}
