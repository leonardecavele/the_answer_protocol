mod app;
mod assets;
mod commands;
mod components;
mod config;
mod events;
mod network;
mod state;

use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use events::AppEvent;
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use state::ConnectionState;
use std::{env, io, sync::Arc, time::Duration};
use tokio::sync::{Mutex, mpsc};

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
    dotenvy::dotenv().ok();
    config::init_config();
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

    while !app.state.should_quit {
        terminal.draw(|f| {
            app.active_component.draw(&mut app.state, f, f.area());

            // Draw notifications as an overlay
            if !app.state.ui.notifications.is_empty() {
                let max_len = app
                    .state
                    .ui
                    .notifications
                    .iter()
                    .map(|n| n.message.len() as u16)
                    .max()
                    .unwrap_or(0);
                let box_width = (max_len + 4).max(20).min(f.area().width.saturating_sub(4));

                let mut total_lines = 0;
                let inner_width = box_width.saturating_sub(2).max(1);
                for n in &app.state.ui.notifications {
                    let lines = (n.message.len() as u16 + inner_width - 1) / inner_width;
                    total_lines += lines.max(1);
                }

                let notif_area = ratatui::layout::Rect {
                    x: f.area().width.saturating_sub(box_width + 1),
                    y: 1,
                    width: box_width,
                    height: total_lines + 2,
                };
                let mut lines = Vec::new();
                for n in &app.state.ui.notifications {
                    let color = match n.level {
                        crate::state::NotificationType::Info => ratatui::style::Color::Cyan,
                        crate::state::NotificationType::Error => ratatui::style::Color::Red,
                    };
                    lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                        n.message.clone(),
                        ratatui::style::Style::default().fg(color),
                    )));
                }
                let notif_block = ratatui::widgets::Paragraph::new(lines)
                    .block(
                        ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                            .title(" Notifications "),
                    )
                    .wrap(ratatui::widgets::Wrap { trim: true });
                f.render_widget(ratatui::widgets::Clear, notif_area);
                f.render_widget(notif_block, notif_area);
            }
        })?;

        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Tick => {
                    if let Some(client_arc) = &app.state.net.client {
                        if let Ok(c) = client_arc.try_lock() {
                            if !c.is_connected() {
                                let _ = tx.send(AppEvent::ApiDisconnected);
                            }
                        }
                    }

                    app.state.ui.notifications.retain_mut(|notif| {
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
                                    app.state.ui.show_debug = !app.state.ui.show_debug;
                                    continue;
                                }
                                KeyCode::Char('h') => {
                                    app.state.ui.show_help = !app.state.ui.show_help;
                                    continue;
                                }
                                KeyCode::Char('t') => {
                                    app.state.ui.show_chat = !app.state.ui.show_chat;
                                    continue;
                                }
                                _ => {}
                            }
                        }

                        // Esc: close overlays or quit
                        if key.code == KeyCode::Esc {
                            if app.state.ui.show_debug || app.state.ui.show_help {
                                app.state.ui.show_debug = false;
                                app.state.ui.show_help = false;
                            } else {
                                app.state.should_quit = true;
                            }
                            continue;
                        }
                    }

                    // Dispatch to active component
                    app.active_component
                        .handle_event(&mut app.state, &evt, &tx)
                        .await;
                }
                AppEvent::AttemptConnect(name, ip, port) => {
                    app.state.ui.push_notification(
                        format!("Connecting to {}:{}...", ip, port),
                        crate::state::NotificationType::Info,
                        10,
                    );
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        use api_client::client::connect::ClientConnect;

                        match ClientConnect::connect(format!("{}:{}", ip, port)).await {
                            Ok(mut client) => {
                                let tx_ev = tx_clone.clone();
                                // Setup listeners
                                client.on_event(move |ev| {
                                    let _ = tx_ev.send(AppEvent::Api(ev));
                                });

                                let _ = tx_clone.send(AppEvent::ClientConnected(client));
                                let _ = tx_clone.send(AppEvent::ExecuteConnect(name));
                            }
                            Err(e) => {
                                log::error!("TCP Connection failed: {}", e);
                                let _ = tx_clone.send(AppEvent::CommandError(format!(
                                    "TCP Connection failed: {}",
                                    e
                                )));
                                let _ = tx_clone.send(AppEvent::ApiDisconnected);
                            }
                        }
                    });
                }
                AppEvent::ExecuteConnect(name) => {
                    if let Some(client_arc) = &app.state.net.client {
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
                    app.state.net.connection_state = ConnectionState::Connected(name.clone());
                    app.state.net.connected_at = Some(std::time::Instant::now());
                    app.switch_to_game();
                    app.state
                        .game
                        .push_game_output(format!("Welcome, {}!", name));

                    if let Some(client_arc) = &app.state.net.client {
                        let client_arc = Arc::clone(client_arc);
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            let c = client_arc.lock().await;
                            if let Ok(Ok(data)) = c.who().await {
                                let _ = tx_clone
                                    .send(AppEvent::UpdateOnlinePlayers(data.player_count as u32));
                            }
                        });
                    }
                }
                AppEvent::CommandResult(res) => {
                    for line in res.lines() {
                        app.state.game.push_game_output(format!("{}", line));
                    }
                }
                AppEvent::CommandError(err) => {
                    for line in err.lines() {
                        app.state.game.push_game_output(format!("[ERROR] {}", line));
                    }
                    app.state
                        .ui
                        .push_notification(err, crate::state::NotificationType::Error, 16);
                }
                AppEvent::ClientConnected(client) => {
                    app.state.net.client = Some(Arc::new(Mutex::new(client)));
                    app.state
                        .game
                        .push_game_output("TCP connection established.".to_string());
                }
                AppEvent::ApiDisconnected => {
                    app.state.net.connection_state = ConnectionState::Disconnected;
                    app.switch_to_login();
                    if app.state.net.client.is_some() {
                        app.state.ui.push_notification(
                            "Disconnected from server".to_string(),
                            crate::state::NotificationType::Error,
                            16,
                        );
                        app.state.net.client = None;
                    }
                }
                AppEvent::InventoryUpdate(items) => {
                    app.state.game.inventory = items;
                }
                AppEvent::UpdateOnlinePlayers(count) => {
                    app.state.game.online_players = count;
                }
                AppEvent::LocalChatSent(scope, msg) => {
                    let sender =
                        if let ConnectionState::Connected(name) = &app.state.net.connection_state {
                            format!("{} (You)", name)
                        } else {
                            "You".to_string()
                        };
                    app.state.game.push_chat(scope, sender, msg);
                }
                AppEvent::UpdateGroup(group_name) => {
                    app.state.game.group_name = group_name;
                }
                AppEvent::UpdateRoomContext {
                    room_id,
                    room_display_name,
                    npcs,
                } => {
                    app.state.game.current_room = room_id;
                    app.state.game.current_room_name = room_display_name;
                    app.state.game.npcs_in_room = npcs;
                }
                AppEvent::UpdateStatus { hp, max_hp } => {
                    app.state.game.hp = hp;
                    app.state.game.max_hp = max_hp;
                }
                AppEvent::Api(api_evt) => {
                    crate::network::handle_server_event(&mut app.state, api_evt);
                }
            }
        }
    }
    Ok(())
}
