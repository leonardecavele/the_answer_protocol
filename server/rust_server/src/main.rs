use rust_server::constants::{AUTO_SAVE_INTERVAL, TickResult};
use rust_server::game_manager::GameManager;
use rust_server::logs::ChannelWriter;
use rust_server::parser::Parser;

use clap::Parser as ClapParser;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, ExternalPrinter};
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use time::macros::format_description;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::LocalTime;

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

fn start_input_reader_thread(
    mut rustyline: DefaultEditor,
    command_sender: mpsc::Sender<String>,
    running: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        loop {
            match rustyline.readline(">>> ") {
                Ok(line) => {
                    if command_sender.send(line).is_err() {
                        break;
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    info!("stopping server...");
                    running.store(false, Ordering::Relaxed);
                    break;
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
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("app.log")
            .ok();

        log_receiver.into_iter().for_each(|msg| {
            if msg == "FLUSH_EXIT" {
                let _ = std::process::Command::new("stty").arg("sane").status();
                std::process::exit(1);
            }
            if let Some(f) = file.as_mut() {
                use std::io::Write;
                let _ = write!(f, "{}", msg);
            }
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

    let printer = rustyline
        .create_external_printer()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let (log_sender, log_receiver) = mpsc::channel::<String>();
    start_log_printer_thread(printer, log_receiver);

    let (command_sender, command_receiver) = mpsc::channel::<String>();

    let time_format = format_description!("[hour]:[minute]:[second].[subsecond digits:6]");
    let timer = LocalTime::new(time_format);
    let log_sender_for_writer = log_sender.clone();
    tracing_subscriber::fmt()
        .with_writer(move || ChannelWriter {
            sender: log_sender_for_writer.clone(),
        })
        .with_env_filter({
            let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
            if let Ok(directive) = "rustyline=warn".parse() {
                filter.add_directive(directive)
            } else {
                filter
            }
        })
        .with_timer(timer)
        .init();

    let mut parser = Parser::new("npcs.json", "items.json", "rooms.json", "quests.json");
    if let Err(e) = parser.parse_all() {
        error!("Parser error: {}", e);
        let _ = log_sender.send("FLUSH_EXIT".to_string());
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    let listener = TcpListener::bind(format!("0.0.0.0:{}", args.rust_server_port))?;
    listener.set_nonblocking(true)?;
    // port can change if it is 0 (takes a free port)
    let port = listener.local_addr()?.port();

    // Channel for the code tester thread to send results back
    let (tester_sender, tester_receiver) = mpsc::channel();

    let running = Arc::new(AtomicBool::new(true));
    let cloned_running = running.clone();

    ctrlc::set_handler(move || {
        info!("stopping server...");
        cloned_running.store(false, Ordering::Relaxed);
    })
    .expect("error while setting Ctrl-C handler");

    start_input_reader_thread(rustyline, command_sender, running.clone());

    let mut game_manager: Option<GameManager> = None;
    let mut tester_receiver = Some(tester_receiver);
    let mut command_receiver = Some(command_receiver);

    'outer: while running.load(Ordering::Relaxed) {
        info!("Game server started (port {})", port);

        let (writer_stream, addr) = loop {
            if !running.load(Ordering::Relaxed) {
                break 'outer;
            }
            match listener.accept() {
                Ok((stream, addr)) => {
                    stream.set_nonblocking(false)?;
                    break (stream, addr);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(_) => break 'outer,
            }
        };
        writer_stream.set_nodelay(true)?;

        info!("Tcp server connected: {}", addr);

        let reader_stream = writer_stream.try_clone()?;

        let (mpsc_sender, mpsc_receiver) = mpsc::channel();

        // channel to make the two threads communicate
        start_tcp_reader_thread(reader_stream, mpsc_sender);

        if let Some(ref mut game_manager) = game_manager {
            game_manager.reset_for_reconnect(writer_stream, mpsc_receiver, &parser);
        } else if let (Some(tester_receiver), Some(command_receiver)) =
            (tester_receiver.take(), command_receiver.take())
        {
            game_manager = Some(GameManager::new(
                mpsc_receiver,
                tester_receiver,
                tester_sender.clone(),
                writer_stream,
                &parser,
                command_receiver,
            ));
        }

        let Some(ref mut game_manager) = game_manager else {
            continue;
        };

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
                break;
            }

            let start = Instant::now(); // this tick time start
            game_manager.update_game_state()?;

            match game_manager.process_incoming_events(start)? {
                TickResult::TickEnd => {
                    game_manager.send_diff_to_players()?;
                    game_manager.clear_diff();
                }
                TickResult::Exit => {
                    info!("Tcp server connection closed");
                    break;
                }
            }
        }

        game_manager.save_server_state();
    }

    return Ok(());
}
