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

pub fn apply_background(frame: &mut Frame) {
    for cell in frame.buffer_mut().content.iter_mut() {
        if cell.bg == Color::Reset {
            cell.bg = BACKGROUND;
        }
    }
}
