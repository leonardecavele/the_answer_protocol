use std::sync::mpsc;
use crate::constantes::{TICK_TIME, TickResult};
use std::time::Instant;
use std::io::Write;
use json::object;
use std::net::TcpStream;
use crate::game_manager::GameManager;

impl GameManager {
    pub fn apply_players_changes(&mut self, mpsc_receiver: &mpsc::Receiver<String>, tick_timer: Instant, writer_stream: &mut TcpStream) -> std::io::Result<TickResult> {
        loop {
            if tick_timer.elapsed() >= TICK_TIME {
                break;
            }
            match mpsc_receiver.recv_timeout(TICK_TIME - tick_timer.elapsed()) {
                Ok(msg) => {
                    let command_response = self.handle_message(msg);
                    writer_stream.write_all(command_response.as_bytes())?;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(TickResult::Exit),
            };
        }
        return Ok(TickResult::TickEnd);
    }

    pub fn update_game_state(&mut self) {
        /*
        here we should update the game state ( npcs, monsters, mouvements, etc 
        and store the diff in a buffer (an argument of this function), 
        we will send it back to players at the end of the tick
        */
    }

    pub fn handle_message(&mut self, msg: String) -> String {
        /*
        read the message, simulate the corresponding action and return the response
        */
        let json_object = json::parse(&msg).unwrap();
        let player_name = json_object["player"].as_str().unwrap();
        let command_name = json_object["command"].as_str().unwrap();
        let command_id = json_object["command_id"].as_str().unwrap();
        // let arguments = json_object["arguments"];
        match command_name {
            "CONNECT" => {
                let error_code = self.init_player(player_name.to_string());
                return object!{
                    "player": player_name,
                    "command_id": command_id,
                    "error_code":error_code.code(),
                    "value": ""
                }.dump();
            }
            // "LOOK" => {},
            // "MOVE" => {},
            // "QUIT" => {},
            // "CHAT" => {},
            // "WHO" => {},
            // "GROUP CREATE" => {},
            // "GROUP INVITE" => {},
            // "GROUP JOIN" => {},
            // "GROUP LEAVE" => {},
            // "TAKE" => {},
            // "DROP" => {},
            // "INVENTORY" => {},
            // "TALK" => {},
            // "ATTACK" => {},
            // "STATUS" => {},
            // "QUEST" => {},
            // "QUESTS" => {},
            _ => {println!("Unknown command: {}", command_name); return "".to_string();}
        }
    }
}
