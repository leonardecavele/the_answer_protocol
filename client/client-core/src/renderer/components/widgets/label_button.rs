use crate::renderer::components::is_mouse_in_rect;
use crate::renderer::theme::WARNING_COLOR;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Clear, Paragraph};

pub struct LabelButton {
    label: String,
    area: Option<Rect>,
}

impl LabelButton {
    pub fn new(label: &str) -> Self {
        Self {
            label: format!(" [{}] ", label),
            area: None,
        }
    }

    pub fn width(&self) -> u16 {
        self.label.chars().count() as u16
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.area = Some(area);

        let style = Style::default()
            .fg(WARNING_COLOR)
            .add_modifier(Modifier::BOLD);

        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new(self.label.as_str()).style(style), area);
    }

    pub fn hide(&mut self) {
        self.area = None;
    }

    pub fn hit(&self, column: u16, row: u16) -> bool {
        self.area
            .is_some_and(|area| is_mouse_in_rect(column, row, area))
    }
}
