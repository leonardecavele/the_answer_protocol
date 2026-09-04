use crate::ui::components::is_mouse_in_rect;
use crate::ui::theme::OVERLAY_BORDER_COLOR;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Clear, Paragraph};

const LABEL: &str = " [X] ";

#[derive(Default)]
pub struct CloseButton {
    area: Option<Rect>,
}

impl CloseButton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, frame: &mut Frame, overlay_area: Rect) {
        let width = LABEL.chars().count() as u16;

        if overlay_area.width <= width {
            self.hide();
            return;
        }

        let area = Rect::new(overlay_area.right() - width - 1, overlay_area.y, width, 1);

        self.area = Some(area);

        let style = Style::default()
            .fg(OVERLAY_BORDER_COLOR)
            .add_modifier(Modifier::BOLD);

        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new(LABEL).style(style), area);
    }

    pub fn hide(&mut self) {
        self.area = None;
    }

    pub fn hit(&self, column: u16, row: u16) -> bool {
        self.area
            .is_some_and(|area| is_mouse_in_rect(column, row, area))
    }
}
