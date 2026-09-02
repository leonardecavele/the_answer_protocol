use rust_server::constants::{AUTO_SAVE_INTERVAL, TickResult};
use rust_server::game_manager::GameManager;
use rust_server::parser::Parser;
use rust_server::logs::ChannelWriter;


use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use time::macros::format_description;
use tracing::{debug, error, info};
use rustyline::{DefaultEditor, ExternalPrinter};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::LocalTime;
use clap::Parser as ClapParser;

fn start_tcp_reader_thread(reader_stream: TcpStream, mpsc_sender: mpsc::Sender<String>) {
    thread::spawn(move || {
        let reader = BufReader::new(reader_stream);

        for line in reader.lines() {
            match line {
                Ok(message) => {
                    let _ = mpsc_sender.send(message);
                }
                Err(err) => {
                    error!("Read error: {}", err);
                    break;
                }
            }
        }
    });
}

fn start_input_reader_thread(mut rustyline: DefaultEditor, command_sender: mpsc::Sender<String>) {
    thread::spawn(move || {
            loop {
                match rustyline.readline(">>> ") {
                    Ok(line) => { 
                        if command_sender.send(line).is_err() { break; }
                    }
                    Err(_) => break,
                }
            }
        });
}

fn start_log_printer_thread<P>(mut printer: P, log_receiver: mpsc::Receiver<String>)
where
    P: ExternalPrinter + Send + 'static,
{
    thread::spawn(move || {
        log_receiver.into_iter().for_each(|msg| {
            printer.print(msg).ok();
        });
    });
}

#[derive(ClapParser)]
struct Args {
    #[arg(long, default_value_t = 38801)]
    rust_server_port: u16,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut rustyline = DefaultEditor::new()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let printer = rustyline.create_external_printer()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    // this channel is used to send logs to 
    let (log_sender, log_receiver) = mpsc::channel::<String>();

    start_log_printer_thread(printer, log_receiver);
    
    //this channel is for receiving commands from the thread that reads the admin input
    let (command_sender, command_receiver) = mpsc::channel::<String>();
    start_input_reader_thread(rustyline, command_sender);
    
    let time_format = format_description!("[hour]:[minute]:[second].[subsecond digits:6]");
    let timer = LocalTime::new(time_format);
    tracing_subscriber::fmt()
        .with_writer(move || ChannelWriter{ sender: log_sender.clone()} )
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
                .add_directive("rustyline=warn".parse().unwrap()),
        )
        
        .with_timer(timer)
        .init();
    
    let mut parser = Parser::new("npcs.json", "items.json", "rooms.json", "quests.json");

    parser.parse_all();

    let listener = TcpListener::bind(format!("0.0.0.0:{}", args.rust_server_port))?;
    let port = listener.local_addr()?.port();
    // port can change if it is 0 (takes a free port)
    info!("Rust server started on {}", port);

    let (writer_stream, addr) = listener.accept()?;
    writer_stream.set_nodelay(true)?;

    info!("Go connected: {}", addr);

    let reader_stream = writer_stream.try_clone()?;

    let (mpsc_sender, mpsc_receiver) = mpsc::channel();

    // Channel for the code tester thread to send results back
    let (tester_sender, tester_receiver) = mpsc::channel();

    // channel to make the two threads communicate
    start_tcp_reader_thread(reader_stream, mpsc_sender);
    let mut game_manager = GameManager::new(
        mpsc_receiver,
        tester_receiver,
        tester_sender,
        writer_stream,
        parser,
        command_receiver,
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
            let now = Instant::now();
            game_manager.save_server_state();
            last_save_time = Instant::now();
            debug!("saved server state in {:?} seconds", now.elapsed());
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
                info!("exiting...");
                game_manager.save_server_state();
                return Ok(());
            }
        }
    }
}
