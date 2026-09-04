use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::{EventFlow, InteractiveComponent, Lifecycle};
use crate::ui::theme::{default_block, dim_style};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use tokio::sync::mpsc::Sender;

#[derive(Default)]
pub struct TextInput {
    pub label: String,
    pub value: String,
    pub is_focused: bool,
}

impl TextInput {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            value: String::new(),
            is_focused: false,
        }
    }
}

impl InteractiveComponent for TextInput {
    fn render(&mut self, _state: &AppState, frame: &mut Frame, area: Rect) {
        let text_style = if self.is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            dim_style()
        };

        let block = default_block()
            .title(format!(" {} ", self.label.as_str()))
            .style(text_style);

        let display_text = if self.is_focused {
            format!("{}█", self.value)
        } else {
            self.value.clone()
        };

        let paragraph = Paragraph::new(display_text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn handle_interactive_event(
        &mut self,
        _state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &Sender<ApplicationEvent>,
        _is_hovered: bool,
    ) -> EventFlow {
        if !self.is_focused {
            return EventFlow::Ignored;
        }

        if let CrosstermEvent::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char(c) => {
                    self.value.push(*c);
                    EventFlow::Consumed
                }
                KeyCode::Backspace => {
                    self.value.pop();
                    EventFlow::Consumed
                }
                _ => EventFlow::Ignored,
            }
        } else {
            EventFlow::Ignored
        }
    }
}

impl Lifecycle for TextInput {}
