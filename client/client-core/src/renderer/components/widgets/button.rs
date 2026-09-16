use crate::events::ApplicationEvent;
use crate::renderer::components::{Component, EventFlow, Lifecycle};
use crate::renderer::theme::{SUCCESS_COLOR, default_block, dim_style};
use crate::states::AppState;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Paragraph;
use tokio::sync::mpsc::Sender;

pub struct Button {
    label: String,
    is_focused: bool,
    is_pressed: bool,
    area: Option<Rect>,
}

impl Button {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            is_focused: false,
            is_pressed: false,
            area: None,
        }
    }

    pub fn hide(&mut self) {
        self.area = None;
    }

    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    pub fn blur(&mut self) {
        self.is_focused = false;
    }

    pub fn press(&mut self) {
        self.is_pressed = true;
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

impl Component for Button {
    fn drawn_area(&self) -> Option<Rect> {
        self.area
    }

    fn draw(&mut self, _state: &AppState, frame: &mut Frame, area: Rect) {
        self.area = Some(area);

        let style = if self.is_focused {
            Style::default()
                .fg(SUCCESS_COLOR)
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
}

impl Lifecycle for Button {
    fn on_key(
        &mut self,
        _state: &mut AppState,
        key: &KeyEvent,
        _sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if !self.is_focused || key.code != KeyCode::Enter {
            return EventFlow::Ignored;
        }

        self.is_pressed = true;

        EventFlow::Consumed
    }
}
