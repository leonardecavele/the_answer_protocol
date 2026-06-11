use std::sync::mpsc;
use crate::{constantes::{TICK_TIME, TickResult}};
use std::time::Instant;
use std::io::Write;
use json::{object, JsonValue};
use std::net::TcpStream;
use crate::game_manager::GameManager;
use tracing::info;

impl GameManager {
    pub fn apply_players_changes(&mut self, mpsc_receiver: &mpsc::Receiver<String>, tick_timer: Instant, writer_stream: &mut TcpStream) -> std::io::Result<TickResult> {
        loop {
            if tick_timer.elapsed() >= TICK_TIME {
                break;
            }
            match mpsc_receiver.recv_timeout(TICK_TIME - tick_timer.elapsed()) {
                Ok(msg) => {
                    let command_response = self.handle_message(msg);
                    writer_stream.write_all((command_response + "\n").as_bytes())?;

                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(TickResult::Exit),
            };
        }
        return Ok(TickResult::TickEnd);
    }

    

    pub fn update_game_state(&mut self, writer_stream: &mut TcpStream) -> std::io::Result<()> {
        /*
        here we should update the game state ( npcs, monsters, mouvements, etc 
        and store the diff in a buffer (an argument of this function), 
        we will send it back to players at the end of the tick
        */


        //hardcoded event for now : 
        
        let event_type = "partouze"; 
        let mut all_events: Vec<JsonValue> = vec![];
        for (player_name, _) in self.get_players_by_names(){
            if player_name == "GABIN" || player_name == "AYMERIC" {
            all_events.push(object!{
                "player": player_name.as_str(),
                "event_name": event_type,
                "value": ""
            });
        }
        }
        let array = JsonValue::Array(all_events);
        if array.is_empty(){
            return Ok(());
        }
        writer_stream.write_all((array.dump() + "\n").as_bytes())?;
        
        info!("sent event");
        Ok(())
    }

}
