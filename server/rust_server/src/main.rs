use rust_server::constantes::{AUTO_SAVE_INTERVAL, TickResult};
use rust_server::game_manager::GameManager;
use rust_server::parser::Parser;

use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use time::macros::format_description;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::LocalTime;

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

    parser.parse_all();

    let listener = TcpListener::bind("0.0.0.0:38801")?;

    println!("Rust server started on 38801");

    let (writer_stream, addr) = listener.accept()?;
    writer_stream.set_nodelay(true)?;

    println!("Go connected: {}", addr);

    let reader_stream = writer_stream.try_clone()?;

    let (mpsc_sender, mpsc_receiver) = mpsc::channel();

    // Channel for the code tester thread to send results back
    let (tester_sender, tester_receiver) = mpsc::channel();

    // channel to make the two threads communicate
    start_reader_thread(reader_stream, mpsc_sender);
    let mut game_manager = GameManager::new(
        mpsc_receiver,
        tester_receiver,
        tester_sender,
        writer_stream,
        parser,
    );

    let running = Arc::new(AtomicBool::new(true));
    let cloned_running = running.clone();

    ctrlc::set_handler(move || {
        info!("stopping server...");
        cloned_running.store(false, Ordering::Relaxed);
    })
    .expect("error while setting Ctrl-C handler");

    let mut last_save_time = Instant::now();
    loop {
        if last_save_time.elapsed() >= AUTO_SAVE_INTERVAL {
            game_manager.save_server_state();
            last_save_time = Instant::now();
            info!("[routine] saved server state");
        }

        if !running.load(Ordering::Relaxed) {
            game_manager.save_server_state();
            return Ok(());
        }

        let start = Instant::now(); // this tick time start
        game_manager.update_game_state()?;

        match game_manager.apply_players_changes(start)? {
            TickResult::TickEnd => {
                game_manager.send_diff_to_players()?;
                game_manager.clear_diff();
            }
            TickResult::Exit => {
                println!("exiting...");
                game_manager.save_server_state();
                return Ok(());
            }
        }
    }
}
