use rust_server::simulation::{apply_players_changes, update_game_state};
use rust_server::player_response::send_diff;
use rust_server::constantes::TickResult;
use std::io::{BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Instant;
use std::sync::mpsc;

fn start_reader_thread(reader_stream: TcpStream, mpsc_sender: mpsc::Sender<String>)
{
    thread::spawn(move || {
        let reader = BufReader::new(reader_stream);

        for line in reader.lines() {
            match line {
                Ok(message) => {
                    let _ = mpsc_sender.send(message);
                }
                Err(err) => {
                    eprintln!("Read error: {}", err);
                    break;
                }
            }
        }
    });
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:38801")?;

    println!("Rust server started on 38801");

    let (mut writer_stream, addr) = listener.accept()?;

    println!("Go connected: {}", addr);

    let reader_stream = writer_stream.try_clone()?;

    let (mpsc_sender, mpsc_receiver) = mpsc::channel();
    // channel to make the two threads communicate: 
    // first thread reads from the socket and sends the message to the receiver
    // the receover now reads the sent message and sends PONG back to the go server
    start_reader_thread(reader_stream, mpsc_sender);

    loop {     
        let start = Instant::now(); // this tick's time start
        /*
         here we should update the game state ( npcs, monsters, mouvements, etc 
         and store the diff in a buffer, we will send it back to players at the end of the tick
        */
        update_game_state();


        match apply_players_changes(&mpsc_receiver, start, &mut writer_stream)? {
            TickResult::TickEnd => {
                send_diff();
            }
            TickResult::Exit => {
                println!("exiting...");
                return Ok(());
            }
        }

    }
}
