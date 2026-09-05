use crate::renderer::components::is_mouse_in_rect;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Clear, Paragraph};

pub struct CommandButton {
    label: String,
    command: &'static str,
    area: Option<Rect>,
}

impl CommandButton {
    pub fn new(label: &str, command: &'static str) -> Self {
        Self {
            label: format!(" [{}] ", label),
            command,
            area: None,
        }
    }

    pub fn width(&self) -> u16 {
        self.label.chars().count() as u16
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.area = Some(area);

        let style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new(self.label.as_str()).style(style), area);
    }

    pub fn hide(&mut self) {
        self.area = None;
    }

    pub fn hit(&self, column: u16, row: u16) -> Option<&'static str> {
        let area = self.area?;

        if is_mouse_in_rect(column, row, area) {
            return Some(self.command);
        }

        None
    }
}
