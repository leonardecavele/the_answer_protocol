use std::sync::mpsc;
use crate::constantes::{TICK_TIME, TickResult};
use std::time::Instant;
use std::io::Write;
use std::net::TcpStream;

pub fn apply_players_changes(mpsc_receiver: &mpsc::Receiver<String>, tick_timer: Instant, writer_stream: &mut TcpStream) -> std::io::Result<TickResult>
{
    loop {
        if tick_timer.elapsed() >= TICK_TIME {
            break;
        }
        match mpsc_receiver.recv_timeout(TICK_TIME - tick_timer.elapsed())
        {
            Ok(msg) =>
            {
                if msg == "PING" {
                    writer_stream.write_all(b"PONG\n")?;
                }
            }

            Err(mpsc::RecvTimeoutError::Timeout) =>
            {
                break;
            }

            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Ok(TickResult::Exit);
            }
        };
    }
    return Ok(TickResult::TickEnd);
}



pub fn update_game_state(){
/*
here we should update the game state ( npcs, monsters, mouvements, etc 
and store the diff in a buffer (an argument of this function), 
we will send it back to players at the end of the tick
*/
}

