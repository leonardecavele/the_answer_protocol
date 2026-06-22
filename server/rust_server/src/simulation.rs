use crate::constantes::{TICK_TIME, TickResult};
use crate::game_manager::GameManager;
use std::sync::mpsc;
use std::time::Instant;
use tracing::info;

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
        /*
        here we should update the game state ( npcs, monsters, mouvements, etc
        and store the diff in a buffer (an argument of this function),
        we will send it back to players at the end of the tick
        */

        //hardcoded event for now :

        // let mut events = json::JsonValue::new_array();
        // let events = array![object! {
        //     "player": "*",
        //     "ignored_players": ["GABIN"],
        //     "emitted_by": "GABIN",
        //     "event_name": "CONNECT",
        //     "data": ""
        // }];


        // self.send_msg_to_client(events.dump().to_string())?;
        // info!("sent event");
        Ok(())
    }
}
