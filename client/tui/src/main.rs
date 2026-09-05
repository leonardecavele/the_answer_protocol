mod app;

use crate::app::TuiApp;
use clap::Parser;
use client_core::{Assets, Cli, logging};
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use std::{io, panic};

const LOG_FILE: &str = "tui.log";

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

    let mut app = TuiApp::new(cli.ip, cli.port, Assets::new(cli.assets))?;
    let res = app.run().await;

    app.restore()?;

    if let Err(err) = res {
        eprintln!("Application Exited with Error: {:?}", err);
    }

    Ok(())
}
