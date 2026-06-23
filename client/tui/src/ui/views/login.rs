use crate::states::app::AppState;
use crate::ui::AppView;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct LoginView {
    // Example: local state for the view
    pub input_buffer: String,
}

impl LoginView {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
        }
    }
}

impl AppView for LoginView {
    fn draw(&mut self, _state: &AppState, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Login to The Answer Protocol")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let text = format!("Please enter your player name:\n\n> {}_", self.input_buffer);

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, _state: &mut AppState, event: &CrosstermEvent) {
        if let CrosstermEvent::Key(key_event) = event {
            match key_event.code {
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Enter => {
                    // TODO: Trigger network login via state change or event bus
                }
                _ => {}
            }
        }
    }
}
