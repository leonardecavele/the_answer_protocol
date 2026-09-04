mod terminal;

use crate::terminal::ClientTerminal;
use eframe::egui;
use tokio::runtime::Handle;
use tui::app::App;
use tui::events::TICK_RATE;

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "38800";

struct GuiApp {
    app: App,
    terminal: ClientTerminal,
    runtime: Handle,
}

impl GuiApp {
    fn new(runtime: Handle) -> Self {
        Self {
            app: App::new(DEFAULT_IP.to_string(), DEFAULT_PORT.to_string()),
            terminal: terminal::build(),
            runtime,
        }
    }
}

impl eframe::App for GuiApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let _guard = self.runtime.enter();

        while let Ok(event) = self.app.try_next_event() {
            self.app.update(event);
        }

        let _ = self.terminal.draw(|frame| {
            self.app.draw(frame);
            terminal::apply_background(frame);
        });

        ctx.request_repaint_after(TICK_RATE);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                ui.add(self.terminal.backend_mut());
            });
    }
}

#[tokio::main]
async fn main() -> eframe::Result {
    let gui = GuiApp::new(Handle::current());

    eframe::run_native(
        "The Answer Protocol",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(gui))),
    )
}
