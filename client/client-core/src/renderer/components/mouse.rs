use crate::collections::Step;
use crossterm::event::MouseEventKind;
use ratatui::layout::Rect;

/// Helper function to check if the mouse coordinates fall within a Rect
pub fn is_mouse_in_rect(col: u16, row: u16, area: Rect) -> bool {
    col >= area.x && col < area.x + area.width && row >= area.y && row < area.y + area.height
}

/// Helper function to read the row a mouse event points at inside a Rect
pub fn hit_row(area: Option<Rect>, column: u16, row: u16) -> Option<usize> {
    let area = area?;

    if !is_mouse_in_rect(column, row, area) {
        return None;
    }

    Some(row.saturating_sub(area.y) as usize)
}

/// Helper function to read the step a mouse wheel event scrolls by
pub fn scroll_direction(kind: MouseEventKind) -> Option<Step> {
    match kind {
        MouseEventKind::ScrollUp => Some(Step::Previous),
        MouseEventKind::ScrollDown => Some(Step::Next),
        _ => None,
    }
}
