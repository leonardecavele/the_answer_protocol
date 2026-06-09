use rust_server::constantes::TickResult;
use rust_server::game_manager::GameManager;
use std::io::{BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Instant;
use std::sync::mpsc;

fn start_reader_thread(reader_stream: TcpStream, mpsc_sender: mpsc::Sender<String>) {
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
    let mut game_manager = GameManager::new();
    loop {     
        let start = Instant::now(); // this tick time start
        game_manager.update_game_state();

        match game_manager.apply_players_changes(&mpsc_receiver, start, &mut writer_stream)? {
            TickResult::TickEnd => {
                game_manager.send_diff_to_players();
            }
            TickResult::Exit => {
                println!("exiting...");
                return Ok(());
            }
        }
    }
}
