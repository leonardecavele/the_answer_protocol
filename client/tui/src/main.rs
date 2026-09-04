use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::{io, panic};
use tui::app::App;
use tui::cli::Cli;
use tui::data::assets::Assets;
use tui::logging;

const LOG_FILE: &str = "tui.log";

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
    logging::setup(LOG_FILE)?;

    let cli = Cli::parse();

    let mut terminal = terminal_setup()?;

    let mut app = App::with_terminal_input(cli.ip, cli.port, Assets::new(cli.assets));
    let res = app.run(&mut terminal).await;

    terminal_restore(terminal)?;

    if let Err(err) = res {
        eprintln!("Application Exited with Error: {:?}", err);
    }

    Ok(())
}
