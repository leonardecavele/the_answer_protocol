use eframe::egui;
use egui_ratatui::RataguiBackend;
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::style::Color;
use soft_ratatui::embedded_graphics_unicodefonts::{mono_9x18_atlas, mono_9x18_bold_atlas};
use soft_ratatui::{EmbeddedGraphics, SoftBackend};

const TEXTURE_NAME: &str = "client_screen";
const INITIAL_COLUMNS: u16 = 120;
const INITIAL_ROWS: u16 = 40;
const BACKGROUND: Color = Color::Rgb(0x00, 0x00, 0x00);

pub type Screen = Terminal<RataguiBackend<EmbeddedGraphics>>;

pub struct Grid {
    area: egui::Rect,
    cell_size: egui::Vec2,
}

impl Grid {
    pub fn new(screen: &Screen, area: egui::Rect) -> Self {
        Self {
            area,
            cell_size: cell_size(screen),
        }
    }

    pub fn cell_at(&self, position: egui::Pos2) -> Option<(u16, u16)> {
        let offset = position - self.area.min;

        if offset.x < 0.0
            || offset.y < 0.0
            || offset.x >= self.area.width()
            || offset.y >= self.area.height()
        {
            return None;
        }

        Some((
            (offset.x / self.cell_size.x) as u16,
            (offset.y / self.cell_size.y) as u16,
        ))
    }
}

pub fn build() -> Screen {
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

pub fn drawable_size(screen: &Screen, ui: &egui::Ui) -> egui::Vec2 {
    let limit = egui::Vec2::splat(ui.ctx().input(|input| input.max_texture_side) as f32);

    ui.available_size().clamp(cell_size(screen), limit)
}

pub fn apply_background(frame: &mut Frame) {
    for cell in frame.buffer_mut().content.iter_mut() {
        if cell.bg == Color::Reset {
            cell.bg = BACKGROUND;
        }
    }
}

fn cell_size(screen: &Screen) -> egui::Vec2 {
    let backend = &screen.backend().soft_backend;

    egui::Vec2::new(backend.char_width as f32, backend.char_height as f32)
}
