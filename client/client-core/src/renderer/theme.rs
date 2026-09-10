use client_api::commands::QuestStatus;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub const ERROR_COLOR: Color = Color::Rgb(0xE0, 0x52, 0x4A);
pub const WARNING_COLOR: Color = Color::Rgb(0xE8, 0xB8, 0x4B);
pub const SUCCESS_COLOR: Color = Color::Rgb(0x5B, 0xC4, 0x6A);
pub const INFORMATION_COLOR: Color = Color::Rgb(0x5A, 0xA9, 0xE6);

pub const PLAYER_COLOR: Color = Color::Rgb(0xC8, 0x8B, 0xE0);
pub const ROOM_COLOR: Color = Color::Rgb(0x4F, 0xD1, 0xC5);
pub const ITEM_COLOR: Color = Color::Rgb(0x4F, 0xD1, 0xC5);
pub const INVITATION_COLOR: Color = Color::Rgb(0xC8, 0x8B, 0xE0);

pub const CHAT_GLOBAL_COLOR: Color = Color::Rgb(0xE8, 0xB8, 0x4B);
pub const CHAT_GROUP_COLOR: Color = Color::Rgb(0x5B, 0xC4, 0x6A);
pub const CHAT_ROOM_COLOR: Color = Color::Rgb(0x4F, 0xD1, 0xC5);
pub const CHAT_PRIVATE_COLOR: Color = Color::Rgb(0xC8, 0x8B, 0xE0);

pub const SURFACE_COLOR: Color = Color::Rgb(0x16, 0x0B, 0x1A);

pub const OVERLAY_BORDER_COLOR: Color = Color::Rgb(0xC8, 0x8B, 0xE0);
pub const FOCUS_BORDER_COLOR: Color = Color::Rgb(0xE8, 0xB8, 0x4B);
pub const TRACE_COLOR: Color = Color::Rgb(0xC8, 0x8B, 0xE0);
pub const MUTED_COLOR: Color = Color::Rgb(0x8B, 0x7F, 0x95);

pub fn dim_style() -> Style {
    Style::default().fg(MUTED_COLOR)
}

pub fn selection_style(color: Color, selected: bool) -> Style {
    let style = Style::default().fg(color);

    if selected {
        style.add_modifier(Modifier::REVERSED)
    } else {
        style
    }
}

pub fn quest_status(status: &QuestStatus) -> (&'static str, Color) {
    match status {
        QuestStatus::InProgress => ("in progress", WARNING_COLOR),
        QuestStatus::Completed => ("completed", SUCCESS_COLOR),
    }
}

pub fn default_block<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(dim_style())
}

pub fn overlay_block<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(OVERLAY_BORDER_COLOR))
}

pub fn panel_block<'a>(title: impl Into<Line<'a>>, focused: bool) -> Block<'a> {
    let block = default_block().title(title);

    if focused {
        block.border_style(Style::default().fg(FOCUS_BORDER_COLOR))
    } else {
        block
    }
}

pub fn popup_block<'a>(title: impl Into<Line<'a>>) -> Block<'a> {
    overlay_block()
        .title(title)
        .style(Style::default().fg(FOCUS_BORDER_COLOR))
}

pub fn close_hint<'a>() -> Paragraph<'a> {
    Paragraph::new(" Press ESC or ENTER to close ")
        .alignment(Alignment::Center)
        .style(dim_style())
}

pub fn help_hint<'a>() -> Line<'a> {
    Line::from(" Press Ctrl + H to open help ").alignment(Alignment::Center)
}

pub fn too_small_hint<'a>(area: Rect, columns: u16, rows: u16) -> Paragraph<'a> {
    Paragraph::new(format!(
        "This window is too small.\n{} x {} — needs {} x {}",
        area.width, area.height, columns, rows
    ))
    .alignment(Alignment::Center)
    .style(Style::default().fg(WARNING_COLOR))
}
