use crate::input;
use crate::screen::{self, Grid, Screen};
use eframe::egui;
use tokio::runtime::Handle;
use tui::app::App;
use tui::events::{ApplicationEvent, TICK_RATE};

pub struct GuiApp {
    app: App,
    screen: Screen,
    runtime: Handle,
    grid: Option<Grid>,
}

impl GuiApp {
    pub fn new(runtime: Handle, screen: Screen, ip: String, port: String) -> Self {
        Self {
            app: App::new(ip, port),
            screen,
            runtime,
            grid: None,
        }
    }
}

impl eframe::App for GuiApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let _guard = self.runtime.enter();

        for event in input::to_crossterm_events(ctx, self.grid.as_ref()) {
            self.app.update(ApplicationEvent::Terminal(event));
        }

        while let Ok(event) = self.app.try_next_event() {
            self.app.update(event);
        }

        if self.app.state.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        let _ = self.screen.draw(|frame| {
            self.app.draw(frame);
            screen::apply_background(frame);
        });

        ctx.request_repaint_after(TICK_RATE);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                let size = screen::drawable_size(&self.screen, ui);

                let response = ui.allocate_ui(size, |ui| ui.add(self.screen.backend_mut()));

                self.grid = Some(Grid::new(&self.screen, response.inner.rect));
            });
    }
}
