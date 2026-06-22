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
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::AppEvent;
use futures::StreamExt;
use log::error;
use ratatui::{backend::CrosstermBackend, Terminal};
use state::ConnectionState;
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
    dotenvy::dotenv().ok();
    config::init_config();
    if let Err(e) = tui_logger::init_logger(log::LevelFilter::Trace) {
        error!("Unable to init tui logger");
        return Err(Box::from(e));
    }
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
                        state::NotificationType::Info => ratatui::style::Color::Cyan,
                        state::NotificationType::Error => ratatui::style::Color::Red,
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
            crate::events::router::route(event, app, &tx).await;
        }
    }
    Ok(())
}
