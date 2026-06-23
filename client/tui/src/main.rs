pub mod app;
pub mod assets;
pub mod commands;
pub mod components;
pub mod config;
pub mod events;
pub mod network;
pub mod state;

use crate::app::App;
use crate::events::{AppEvent, UiEvent};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::time::Duration;
use std::{io, panic};
use tokio::sync::mpsc;

const TICK_RATE: Duration = Duration::from_millis(500);
const MAX_EVENTS_BUS: usize = 250;

fn terminal_setup() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn terminal_restore(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn setup_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_panic_hook();

    let mut terminal = terminal_setup()?;

    let mut app = App::new("127.0.0.1".to_string(), "38800".to_string());
    let res = app.run(&mut terminal).await;

    terminal_restore(terminal)?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}
