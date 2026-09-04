use eframe::egui::{Ui, Vec2};
use egui_ratatui::RataguiBackend;
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::style::Color;
use soft_ratatui::embedded_graphics_unicodefonts::{mono_9x18_atlas, mono_9x18_bold_atlas};
use soft_ratatui::{EmbeddedGraphics, SoftBackend};

const TEXTURE_NAME: &str = "client_terminal";
const INITIAL_COLUMNS: u16 = 120;
const INITIAL_ROWS: u16 = 40;
const BACKGROUND: Color = Color::Rgb(0x00, 0x00, 0x00);

pub type ClientTerminal = Terminal<RataguiBackend<EmbeddedGraphics>>;

pub fn build() -> ClientTerminal {
    let soft_backend = SoftBackend::<EmbeddedGraphics>::new(
        INITIAL_COLUMNS,
        INITIAL_ROWS,
        mono_9x18_atlas(),
        Some(mono_9x18_bold_atlas()),
        None,
    );

    Terminal::new(RataguiBackend::new(TEXTURE_NAME, soft_backend))
        .expect("a software backend cannot fail to initialise")
}

pub fn drawable_size(terminal: &ClientTerminal, ui: &Ui) -> Vec2 {
    let backend = &terminal.backend().soft_backend;
    let cell = Vec2::new(backend.char_width as f32, backend.char_height as f32);
    let limit = Vec2::splat(ui.ctx().input(|input| input.max_texture_side) as f32);

    ui.available_size().clamp(cell, limit)
}

pub fn apply_background(frame: &mut Frame) {
    for cell in frame.buffer_mut().content.iter_mut() {
        if cell.bg == Color::Reset {
            cell.bg = BACKGROUND;
        }
    }
}
