mod app;
mod events;
mod ui;

use app::{App, ConnectionState};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::AppEvent;
use futures::{FutureExt, StreamExt};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io, sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};

const LOCAL_SERVER_IP: &str = "127.0.0.1";
const LOCAL_SERVER_PORT: &str = "38800";

fn get_server_ip() -> String {
    env::var("SERVER_IP").unwrap_or_else(|_| LOCAL_SERVER_IP.to_string())
}

fn get_server_port() -> String {
    env::var("SERVER_PORT").unwrap_or_else(|_| LOCAL_SERVER_PORT.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup tracing with tui-logger
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);
    
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    tracing_subscriber::registry()
        .with(tui_logger::TuiTracingSubscriberLayer)
        .init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(get_server_ip(), get_server_port());
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Spawn crossterm event reader
    let tick_rate = Duration::from_millis(250);
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut reader = crossterm::event::EventStream::new();
        loop {
            let delay = tokio::time::sleep(tick_rate);
            tokio::select! {
                _ = delay => {
                    let _ = tx_clone.send(AppEvent::Tick);
                }
                Some(Ok(evt)) = reader.next() => {
                    let _ = tx_clone.send(AppEvent::TerminalEvent(evt));
                }
            }
        }
    });

    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Tick => {
                    if let Some(client_arc) = &app.client {
                        if let Ok(c) = client_arc.try_lock() {
                            if !c.is_connected() {
                                let _ = tx.send(AppEvent::ApiDisconnected);
                            }
                        }
                    }
                }
                AppEvent::TerminalEvent(evt) => {
                    use crate::app::Focus;
                    match evt {
                        Event::Key(key) => {
                            match key.code {
                                KeyCode::Esc => app.should_quit = true,
                                KeyCode::Tab => {
                                    app.focus = match app.focus {
                                        Focus::Input => Focus::GameEvents,
                                        Focus::GameEvents => Focus::SystemLogs,
                                        Focus::SystemLogs => Focus::Input,
                                    };
                                }
                                KeyCode::Up => match app.focus {
                                    Focus::GameEvents | Focus::Input => app.scroll_offset = app.scroll_offset.saturating_add(1),
                                    Focus::SystemLogs => app.logger_state.transition(tui_logger::TuiWidgetEvent::UpKey),
                                },
                                KeyCode::Down => match app.focus {
                                    Focus::GameEvents | Focus::Input => app.scroll_offset = app.scroll_offset.saturating_sub(1),
                                    Focus::SystemLogs => app.logger_state.transition(tui_logger::TuiWidgetEvent::DownKey),
                                },
                                KeyCode::PageUp => match app.focus {
                                    Focus::GameEvents | Focus::Input => app.scroll_offset = app.scroll_offset.saturating_add(5),
                                    Focus::SystemLogs => app.logger_state.transition(tui_logger::TuiWidgetEvent::PrevPageKey),
                                },
                                KeyCode::PageDown => match app.focus {
                                    Focus::GameEvents | Focus::Input => app.scroll_offset = app.scroll_offset.saturating_sub(5),
                                    Focus::SystemLogs => app.logger_state.transition(tui_logger::TuiWidgetEvent::NextPageKey),
                                },
                                KeyCode::Enter => {
                                    let cmd_str = app.input.value().to_string();
                                    app.input.reset();
                                    handle_command(app, cmd_str, tx.clone()).await;
                                }
                                _ => {
                                    if matches!(app.focus, Focus::Input) {
                                        tui_input::backend::crossterm::EventHandler::handle_event(&mut app.input, &Event::Key(key));
                                    } else if matches!(app.focus, Focus::SystemLogs) {
                                        match key.code {
                                            KeyCode::Char(' ') => app.logger_state.transition(tui_logger::TuiWidgetEvent::SpaceKey),
                                            KeyCode::Left => app.logger_state.transition(tui_logger::TuiWidgetEvent::LeftKey),
                                            KeyCode::Right => app.logger_state.transition(tui_logger::TuiWidgetEvent::RightKey),
                                            KeyCode::Esc => app.logger_state.transition(tui_logger::TuiWidgetEvent::EscapeKey),
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        Event::Mouse(mouse) => {
                            use crossterm::event::MouseEventKind;
                            match mouse.kind {
                                MouseEventKind::ScrollUp => match app.focus {
                                    Focus::GameEvents | Focus::Input => app.scroll_offset = app.scroll_offset.saturating_add(3),
                                    Focus::SystemLogs => app.logger_state.transition(tui_logger::TuiWidgetEvent::UpKey),
                                },
                                MouseEventKind::ScrollDown => match app.focus {
                                    Focus::GameEvents | Focus::Input => app.scroll_offset = app.scroll_offset.saturating_sub(3),
                                    Focus::SystemLogs => app.logger_state.transition(tui_logger::TuiWidgetEvent::DownKey),
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                AppEvent::ExecuteConnect(name) => {
                    if let Some(client_arc) = &app.client {
                        let client_arc = Arc::clone(client_arc);
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            let mut c = client_arc.lock().await;
                            match c.connect(name.clone()).await {
                                Ok(Ok(_)) => {
                                    log::info!("Connected to game as {}", name);
                                    let _ = tx_clone.send(AppEvent::LoginSuccess(name));
                                }
                                Ok(Err(err)) => {
                                    log::error!("Game connect command failed: {:?}", err);
                                    let _ = tx_clone.send(AppEvent::CommandError(format!("Login failed: {:?}", err)));
                                    let _ = tx_clone.send(AppEvent::ApiDisconnected);
                                }
                                Err(err) => {
                                    log::error!("Network error on connect: {:?}", err);
                                    let _ = tx_clone.send(AppEvent::CommandError(format!("Network error: {:?}", err)));
                                    let _ = tx_clone.send(AppEvent::ApiDisconnected);
                                }
                            }
                        });
                    }
                }
                AppEvent::LoginSuccess(name) => {
                    app.state = ConnectionState::Connected(name.clone());
                    app.push_message(format!("Successfully logged in as {}.", name));
                }
                AppEvent::CommandResult(res) => {
                    for line in res.lines() {
                        app.push_message(format!("[OK] {}", line));
                    }
                }
                AppEvent::CommandError(err) => {
                    for line in err.lines() {
                        app.push_message(format!("[ERROR] {}", line));
                    }
                }
                AppEvent::ClientConnected(client) => {
                    app.client = Some(Arc::new(Mutex::new(client)));
                    app.push_message("TCP Connection established.".to_string());
                }
                AppEvent::ApiDisconnected => {
                    app.state = ConnectionState::Disconnected;
                    app.client = None;
                    app.push_message("Disconnected from server.".to_string());
                }
                AppEvent::Api(api_evt) => {
                    use api_client::client::event::*;
                    match api_evt {
                        ServerEvent::Connect(name) => {
                            if matches!(app.state, ConnectionState::Connecting) {
                                app.state = ConnectionState::Connected(name.clone());
                            }
                            app.push_message(format!("-> {} connected to the server.", name));
                        }
                        ServerEvent::Quit(name) => app.push_message(format!("<- {} quit the server.", name)),
                        ServerEvent::Room(RoomEvent::PresenceEnter(name)) => app.push_message(format!("[Room] {} entered.", name)),
                        ServerEvent::Room(RoomEvent::PresenceLeave(name)) => app.push_message(format!("[Room] {} left.", name)),
                        ServerEvent::Room(RoomEvent::Chat(msg)) => app.push_message(format!("[Room] {}: {}", msg.sender, msg.message)),
                        ServerEvent::Group(GroupEvent::Chat(msg)) => app.push_message(format!("[Group] {}: {}", msg.sender, msg.message)),
                        ServerEvent::Group(GroupEvent::Invite(name)) => app.push_message(format!("[Group] {} invited you.", name)),
                        ServerEvent::Group(GroupEvent::Join(name)) => app.push_message(format!("[Group] {} joined.", name)),
                        ServerEvent::Group(GroupEvent::Leave(name)) => app.push_message(format!("[Group] {} left.", name)),
                        ServerEvent::GlobalChat(msg) => app.push_message(format!("[Global] {}: {}", msg.sender, msg.message)),
                        ServerEvent::PrivateChat(msg) => app.push_message(format!("[Private] {}: {}", msg.sender, msg.message)),
                        ServerEvent::Stats(count) => app.push_message(format!("Server stats: {} players online.", count)),
                        ServerEvent::Unknown(u) => app.push_message(format!("Unknown event: {}", u)),
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_command(app: &mut App, cmd_line: String, tx: mpsc::UnboundedSender<AppEvent>) {
    let parts: Vec<&str> = cmd_line.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd = parts[0];
    app.push_message(format!("> {}", cmd_line));

    if cmd == "quit" {
        app.should_quit = true;
        return;
    }

    if cmd == "connect" && parts.len() == 2 {
        let name = parts[1].to_string();
        let ip = app.server_ip.clone();
        let port = app.server_port.clone();

        app.state = ConnectionState::Connecting;
        app.push_message(format!("Connecting to {}:{}...", ip, port));

        let tx_clone = tx.clone();
        tokio::spawn(async move {
            use api_client::client::connect::ClientConnect;

            match ClientConnect::connect(format!("{}:{}", ip, port)).await {
                Ok(mut client) => {
                    let tx_ev = tx_clone.clone();
                    // Setup listeners
                    client.on_event(move |ev| { let _ = tx_ev.send(AppEvent::Api(ev)); });

                    let _ = tx_clone.send(AppEvent::ClientConnected(client));
                    let _ = tx_clone.send(AppEvent::ExecuteConnect(name));
                }
                Err(e) => {
                    log::error!("TCP Connection failed: {}", e);
                    let _ = tx_clone.send(AppEvent::ApiDisconnected);
                }
            }
        });
        return;
    }

    // Other commands require a client
    if let Some(client_arc) = &app.client {
        let client_arc = Arc::clone(client_arc);
        let cmd = cmd.to_string();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let mut c = client_arc.lock().await;

            macro_rules! handle_res {
                ($res:expr) => {
                    match $res {
                        Ok(Ok(data)) => { let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", data))); }
                        Ok(Err(e)) => { let _ = tx_clone.send(AppEvent::CommandError(format!("Command Error: {:?}", e))); }
                        Err(e) => { let _ = tx_clone.send(AppEvent::CommandError(format!("Network Error: {:?}", e))); }
                    }
                }
            }

            match cmd.as_str() {
                "look" => handle_res!(c.look().await),
                "who" => handle_res!(c.who().await),
                "chat_global" => handle_res!(c.chat_global(args.join(" ")).await),
                "chat_private" if args.len() >= 2 => handle_res!(c.chat_private(args[0].clone(), args[1..].join(" ")).await),
                "group_create" => handle_res!(c.group_create().await),
                "group_invite" if args.len() == 1 => handle_res!(c.group_invite(args[0].clone()).await),
                "group_join" if args.len() == 1 => handle_res!(c.group_join(args[0].clone()).await),
                "group_leave" => handle_res!(c.group_leave().await),
                "take" if args.len() == 1 => handle_res!(c.take(args[0].clone()).await),
                "drop" if args.len() == 1 => handle_res!(c.drop_item(args[0].clone()).await),
                "inventory" => handle_res!(c.inventory().await),
                "talk" if args.len() == 1 => handle_res!(c.talk(args[0].clone()).await),
                "attack" if args.len() == 1 => handle_res!(c.attack(args[0].clone()).await),
                "status" => handle_res!(c.status().await),
                "quest" if args.len() == 1 => handle_res!(c.quest(args[0].clone()).await),
                "quests" => handle_res!(c.quests().await),
                _ => {
                    let _ = tx_clone.send(AppEvent::CommandError(format!("Unknown or malformed command: {}", cmd)));
                }
            }
        });
    } else {
        app.push_message("Error: Not connected. Type `connect <name>` first.".to_string());
    }
}
