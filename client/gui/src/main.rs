mod app;
mod input;
mod terminal;

use crate::app::GuiApp;
use tokio::runtime::Handle;

#[tokio::main]
async fn main() -> eframe::Result {
    let gui = GuiApp::new(Handle::current());

    eframe::run_native(
        "The Answer Protocol",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(gui))),
    )
}
