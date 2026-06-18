mod app;
mod commands;
mod events;
mod handlers;
mod ui;

use app::{App, ChatScope, ConnectionState, Screen};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use events::AppEvent;
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io, sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};

const LOCAL_SERVER_IP: &str = "10.13.6.2";
const LOCAL_SERVER_PORT: &str = "38800";

fn get_server_ip() -> String {
    env::var("SERVER_IP").unwrap_or_else(|_| LOCAL_SERVER_IP.to_string())
}

fn get_server_port() -> String {
    env::var("SERVER_PORT").unwrap_or_else(|_| LOCAL_SERVER_PORT.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    tracing_subscriber::registry()
        .with(tui_logger::TuiTracingSubscriberLayer)
        .init();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(get_server_ip(), get_server_port());
    let res = run_app(&mut terminal, &mut app).await;

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

                    // Decrement notification lifetimes
                    app.notifications.retain_mut(|notif| {
                        notif.lifetime = notif.lifetime.saturating_sub(1);
                        notif.lifetime > 0
                    });
                }
                AppEvent::TerminalEvent(evt) => {
                    if let Event::Key(key) = evt {
                        // Global Ctrl shortcuts
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            match key.code {
                                KeyCode::Char('d') => {
                                    app.show_debug = !app.show_debug;
                                    continue;
                                }
                                KeyCode::Char('h') => {
                                    app.show_help = !app.show_help;
                                    continue;
                                }
                                KeyCode::Char('t') => {
                                    app.show_chat = !app.show_chat;
                                    continue;
                                }
                                _ => {}
                            }
                        }

                        // Esc: close overlays or quit
                        if key.code == KeyCode::Esc {
                            if app.show_debug || app.show_help {
                                app.show_debug = false;
                                app.show_help = false;
                            } else {
                                app.should_quit = true;
                            }
                            continue;
                        }

                        // Screen-specific handling
                        match app.screen {
                            Screen::Login => handlers::login::handle_key(app, key, &evt, &tx),
                            Screen::Game => handlers::game::handle_key(app, key, &evt, &tx).await,
                        }
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
                                    log::info!("Connected as {}", name);
                                    let _ = tx_clone.send(AppEvent::LoginSuccess(name));
                                }
                                Ok(Err(err)) => {
                                    log::error!("Connect failed: {:?}", err);
                                    let _ = tx_clone.send(AppEvent::CommandError(format!(
                                        "Login failed: {:?}",
                                        err
                                    )));
                                    let _ = tx_clone.send(AppEvent::ApiDisconnected);
                                }
                                Err(err) => {
                                    log::error!("Network error: {:?}", err);
                                    let _ = tx_clone.send(AppEvent::CommandError(format!(
                                        "Network error: {:?}",
                                        err
                                    )));
                                    let _ = tx_clone.send(AppEvent::ApiDisconnected);
                                }
                            }
                        });
                    }
                }
                AppEvent::LoginSuccess(name) => {
                    app.state = ConnectionState::Connected(name.clone());
                    app.screen = Screen::Game;
                    app.push_game_output(format!("Welcome, {}!", name));

                    if let Some(client_arc) = &app.client {
                        let client_arc = Arc::clone(client_arc);
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            let c = client_arc.lock().await;
                            if let Ok(Ok(data)) = c.who().await {
                                let _ = tx_clone.send(AppEvent::UpdateOnlinePlayers(data.player_count as u32));
                            }
                        });
                    }
                }
                AppEvent::CommandResult(res) => {
                    for line in res.lines() {
                        app.push_game_output(format!("{}", line));
                    }
                }
                AppEvent::CommandError(err) => {
                    for line in err.lines() {
                        app.push_game_output(format!("[ERROR] {}", line));
                    }
                    app.push_notification(err, crate::app::NotificationType::Error, 16);
                }
                AppEvent::ClientConnected(client) => {
                    app.client = Some(Arc::new(Mutex::new(client)));
                    app.push_game_output("TCP connection established.".to_string());
                    app.push_notification("TCP connected.".to_string(), crate::app::NotificationType::Info, 12);
                }
                AppEvent::ApiDisconnected => {
                    app.state = ConnectionState::Disconnected;
                    app.screen = Screen::Login;
                    app.client = None;
                    app.push_notification("Disconnected from server".to_string(), crate::app::NotificationType::Error, 16);
                }
                AppEvent::InventoryUpdate(items) => {
                    app.inventory = items;
                }
                AppEvent::UpdateOnlinePlayers(count) => {
                    app.online_players = count;
                }
                AppEvent::LocalChatSent(scope, msg) => {
                    let sender = if let ConnectionState::Connected(name) = &app.state {
                        format!("{} (You)", name)
                    } else {
                        "You".to_string()
                    };
                    app.push_chat(scope, sender, msg);
                }
                AppEvent::Api(api_evt) => {
                    use api_client::client::event::*;
                    match api_evt {
                        ServerEvent::Connect(name) => {
                            app.push_game_output(format!("-> {} connected.", name));
                            app.online_players += 1;
                            app.push_notification(format!("{} connected", name), crate::app::NotificationType::Info, 16);
                        }
                        ServerEvent::Quit(name) => {
                            app.push_game_output(format!("<- {} quit.", name));
                            app.online_players = app.online_players.saturating_sub(1);
                            app.push_notification(format!("{} disconnected", name), crate::app::NotificationType::Info, 16);
                        }
                        ServerEvent::Room(RoomEvent::PresenceEnter(name)) => {
                            app.push_game_output(format!("[Room] {} entered.", name));
                        }
                        ServerEvent::Room(RoomEvent::PresenceLeave(name)) => {
                            app.push_game_output(format!("[Room] {} left.", name));
                        }
                        ServerEvent::Room(RoomEvent::Chat(msg)) => {
                            app.push_chat(ChatScope::Room, msg.sender, msg.message);
                        }
                        ServerEvent::Group(GroupEvent::Chat(msg)) => {
                            app.push_chat(ChatScope::Group, msg.sender, msg.message);
                        }
                        ServerEvent::Group(GroupEvent::Invite(name)) => {
                            app.push_game_output(format!("[Group] {} invited you.", name));
                            app.push_notification(format!("{} invited you to group", name), crate::app::NotificationType::Info, 20);
                        }
                        ServerEvent::Group(GroupEvent::Join(name)) => {
                            app.push_game_output(format!("[Group] {} joined.", name));
                        }
                        ServerEvent::Group(GroupEvent::Leave(name)) => {
                            app.push_game_output(format!("[Group] {} left.", name));
                        }
                        ServerEvent::GlobalChat(msg) => {
                            app.push_chat(ChatScope::Global, msg.sender, msg.message);
                        }
                        ServerEvent::PrivateChat(msg) => {
                            app.push_chat(ChatScope::Private, msg.sender, msg.message);
                        }
                        ServerEvent::Stats(count) => {
                            app.online_players = count;
                        }
                        ServerEvent::Unknown(u) => {
                            app.push_game_output(format!("Unknown event: {}", u));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
