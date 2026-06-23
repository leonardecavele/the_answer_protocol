use crate::states::app::AppState;
use crate::ui::components::Component;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use ratatui::Frame;

pub struct TextInputComponent {
    pub label: String,
    pub value: String,
    pub is_focused: bool,
    pub last_area: Option<Rect>,
}

impl TextInputComponent {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            value: String::new(),
            is_focused: false,
            last_area: None,
        }
    }
}

impl Component for TextInputComponent {
    fn is_clickable(&self) -> bool { true }
    
    fn get_last_area(&self) -> Option<Rect> { self.last_area }
    
    fn set_last_area(&mut self, area: Rect) { self.last_area = Some(area); }

    fn draw(&mut self, _state: &AppState, frame: &mut Frame, area: Rect) {
        self.set_last_area(area);
        
        let border_color = if self.is_focused { Color::Cyan } else { Color::DarkGray };
        
        let block = Block::default()
            .title(format!(" {} ", self.label.as_str()))
            .borders(Borders::ALL)
            .style(Style::default().fg(border_color));

        let display_text = if self.is_focused {
            format!("{}█", self.value)
        } else {
            self.value.clone()
        };

        let paragraph = Paragraph::new(display_text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, _state: &mut AppState, event: &CrosstermEvent) -> bool {
        if !self.is_focused {
            return false;
        }

        if let CrosstermEvent::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char(c) => {
                    self.value.push(*c);
                    true
                }
                KeyCode::Backspace => {
                    self.value.pop();
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
}
