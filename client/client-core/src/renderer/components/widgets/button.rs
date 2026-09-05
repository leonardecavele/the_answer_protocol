use crate::renderer::components::{EventFlow, InteractiveComponent, Lifecycle};
use crate::renderer::theme::{default_block, dim_style};
use crate::states::AppState;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Paragraph;
use tokio::sync::mpsc::Sender;

pub struct Button {
    pub label: String,
    pub is_focused: bool,
    pub is_pressed: bool,
    pub last_area: Option<Rect>,
}

impl Button {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            is_focused: false,
            is_pressed: false,
            last_area: None,
        }
    }

    /// Returns true if the button was just pressed, and resets the pressed state.
    pub fn take_pressed(&mut self) -> bool {
        if self.is_pressed {
            self.is_pressed = false;
            true
        } else {
            false
        }
    }
}

impl InteractiveComponent for Button {
    fn render(&mut self, _state: &AppState, frame: &mut Frame, area: Rect) {
        let style = if self.is_focused {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            dim_style()
        };

        let block = default_block().style(style);

        let display_text = if self.is_focused {
            format!("> {} <", self.label)
        } else {
            format!("  {}  ", self.label)
        };

        let paragraph = Paragraph::new(display_text)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn handle_interactive_event(
        &mut self,
        _state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &Sender<crate::events::ApplicationEvent>,
        _is_hovered: bool,
    ) -> EventFlow {
        if !self.is_focused {
            return EventFlow::Ignored;
        }

        if let CrosstermEvent::Key(KeyEvent { code, .. }) = event
            && *code == KeyCode::Enter
        {
            self.is_pressed = true;
            return EventFlow::Consumed;
        }

        EventFlow::Ignored
    }
}

impl Lifecycle for Button {}
