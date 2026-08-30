use api_client::commands::QuestStatus;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub const OVERLAY_BORDER_COLOR: Color = Color::Magenta;
pub const FOCUS_BORDER_COLOR: Color = Color::Yellow;

pub fn dim_style() -> Style {
    Style::default().add_modifier(Modifier::DIM)
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
        QuestStatus::InProgress => ("in progress", Color::Yellow),
        QuestStatus::Completed => ("completed", Color::Green),
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
