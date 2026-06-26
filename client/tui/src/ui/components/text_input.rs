use crate::states::app::AppState;
use crate::ui::components::interactive::InteractiveComponent;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct TextInputComponent {
    pub label: String,
    pub value: String,
    pub is_focused: bool,
}

impl TextInputComponent {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            value: String::new(),
            is_focused: false,
        }
    }
}

impl InteractiveComponent for TextInputComponent {
    fn render(&mut self, _state: &AppState, frame: &mut Frame, area: Rect) {
        let text_color = if self.is_focused {
            Color::Cyan
        } else {
            Color::Gray
        };

        let block = crate::ui::theme::default_block()
            .title(format!(" {} ", self.label.as_str()))
            .style(Style::default().fg(text_color));

        let display_text = if self.is_focused {
            format!("{}█", self.value)
        } else {
            self.value.clone()
        };

        let paragraph = Paragraph::new(display_text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn handle_terminal_event(
        &mut self,
        _state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &tokio::sync::mpsc::Sender<crate::events::ApplicationEvent>,
        _is_hovered: bool,
    ) -> bool {
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
