mod app;
mod input;
mod screen;

use crate::app::GuiApp;
use clap::Parser;
use client_core::Assets;
use client_core::Cli;
use client_core::logging;
use eframe::egui;
use tokio::runtime::Handle;
use tui::ui::{MIN_COLUMNS, MIN_ROWS};

const LOG_FILE: &str = "gui.log";
const WINDOW_TITLE: &str = "The Answer Protocol";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup(LOG_FILE)?;

    let cli = Cli::parse();
    let screen = screen::build();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size(screen::grid_size(
                &screen,
                screen::INITIAL_COLUMNS,
                screen::INITIAL_ROWS,
            ))
            .with_min_inner_size(screen::grid_size(&screen, MIN_COLUMNS, MIN_ROWS)),
        ..Default::default()
    };

    let gui = GuiApp::new(
        Handle::current(),
        screen,
        cli.ip,
        cli.port,
        Assets::new(cli.assets),
    );

    eframe::run_native(WINDOW_TITLE, options, Box::new(|_cc| Ok(Box::new(gui))))?;

    Ok(())
}
