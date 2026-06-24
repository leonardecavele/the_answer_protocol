use rust_server::constantes::TickResult;
use rust_server::game_manager::GameManager;
use rust_server::parser::Parser;

use std::io::{BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Instant;
use std::sync::mpsc;
use time::macros::format_description;
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::EnvFilter;
use tracing::{error, info};

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
    let time_format = format_description!("[hour]:[minute]:[second].[subsecond digits:6]");
    let timer = LocalTime::new(time_format);
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_timer(timer)
        .init();
    
    let mut parser = Parser::new("npcs.json", "items.json", "rooms.json", "quests.json");
    
    match parser.parse_npcs(){
        Ok(()) => {
            info!("Successfully loaded {} NPCs.", parser.get_npcs().len());
        }
        Err(err) => {
            error!("Error parsing NPCs: {}", err);
        }
    }
    
    match parser.parse_items(){
        Ok(()) => {
            info!("Successfully loaded {} items.", parser.get_items().len());
        }
        Err(err) => {
            error!("Error parsing items: {}", err);
        }
    }

    match parser.parse_rooms(){
        Ok(()) => {
            info!("Successfully loaded {} rooms.", parser.get_rooms().len());
        }
        Err(err) => {
            error!("Error parsing rooms: {}", err);
        }
    }

    match parser.parse_quests(){
        Ok(()) => {
            info!("Successfully loaded {} quests.", parser.get_quests().len());
        }
        Err(err) => {
            error!("Error parsing quests: {}", err);
        }
    }

    let listener = TcpListener::bind("0.0.0.0:38801")?;

    println!("Rust server started on 38801");

    let (writer_stream, addr) = listener.accept()?;
    writer_stream.set_nodelay(true)?;
    
    println!("Go connected: {}", addr);

    let reader_stream = writer_stream.try_clone()?;

    let (mpsc_sender, mpsc_receiver) = mpsc::channel();

    // channel to make the two threads communicate: 
    // first thread reads from the socket and sends the message to the receiver
    // the receover now reads the sent message and sends PONG back to the go server
    start_reader_thread(reader_stream, mpsc_sender);
    let mut game_manager = GameManager::new(mpsc_receiver, writer_stream, parser);
    loop {
        let start = Instant::now(); // this tick time start
        game_manager.update_game_state()?;

        match game_manager.apply_players_changes(start)? {
            TickResult::TickEnd => {
                game_manager.send_diff_to_players()?;
                game_manager.clear_diff();
            }
            TickResult::Exit => {
                println!("exiting...");
                return Ok(());
            }
        }
    }
}
