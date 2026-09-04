use crate::input;
use crate::terminal;
use crate::terminal::ClientTerminal;
use eframe::egui;
use tokio::runtime::Handle;
use tui::app::App;
use tui::events::{ApplicationEvent, TICK_RATE};

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "38800";

pub struct GuiApp {
    app: App,
    terminal: ClientTerminal,
    runtime: Handle,
}

impl GuiApp {
    pub fn new(runtime: Handle) -> Self {
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

        for event in input::terminal_events(ctx) {
            self.app.update(ApplicationEvent::Terminal(event));
        }

        while let Ok(event) = self.app.try_next_event() {
            self.app.update(event);
        }

        if self.app.state.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
