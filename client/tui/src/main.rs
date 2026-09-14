mod app;

use crate::app::TuiApp;
use clap::Parser;
use client_core::logging::DeferredErrors;
use client_core::{Assets, Cli, logging};
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use std::io::IsTerminal;
use std::{io, panic, process};

const LOG_FILE: &str = "tui.log";

fn setup_panic_hook(deferred_errors: DeferredErrors) {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        let _ = deferred_errors.flush();
        original_hook(panic_info);
    }));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !io::stdout().is_terminal() {
        eprintln!("This program requires a terminal. Use the GUI client instead.");
        process::exit(1);
    }

    let deferred_errors = logging::setup(LOG_FILE)?;
    setup_panic_hook(deferred_errors.clone());

    let cli = Cli::parse();

    let mut app = TuiApp::new(cli.ip, cli.port, Assets::new(cli.assets))?;
    let res = app.run().await;

    app.restore()?;
    let _ = deferred_errors.flush();

    if let Err(err) = res {
        eprintln!("Application Exited with Error: {:?}", err);
    }

    Ok(())
}
