use ratatui::{
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};

pub const BORDER_COLOR: Color = Color::Gray;
pub const OVERLAY_BORDER_COLOR: Color = Color::Magenta;

pub fn default_block<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR))
}

pub fn overlay_block<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(OVERLAY_BORDER_COLOR))
}
