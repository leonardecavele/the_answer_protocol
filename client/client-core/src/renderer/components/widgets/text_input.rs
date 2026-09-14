use crate::events::ApplicationEvent;
use crate::renderer::components::{EventFlow, InteractiveComponent, Lifecycle};
use crate::renderer::theme::{ITEM_COLOR, default_block, dim_style};
use crate::states::AppState;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;

const CURSOR_BLINK_DURATION: Duration = Duration::from_millis(500);

struct Cursor {
    index: usize,
    is_visible: bool,
    last_blink_instant: Instant,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            index: 0,
            is_visible: true,
            last_blink_instant: Instant::now(),
        }
    }
}

#[derive(Default)]
pub struct TextInput {
    pub label: String,
    pub value: String,
    pub is_focused: bool,
    cursor: Cursor,
    offset: usize,
}

impl TextInput {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            value: String::new(),
            is_focused: false,
            cursor: Cursor::default(),
            offset: 0,
        }
    }

    fn clamp_offset(&mut self, visible_count: usize) {
        if visible_count == 0 {
            self.offset = 0;
            return;
        }

        if self.cursor.index < self.offset {
            self.offset = self.cursor.index;
        } else if self.cursor.index >= self.offset + visible_count {
            self.offset = self.cursor.index - visible_count + 1;
        }
    }

    pub fn cursor_to_end(&mut self) {
        self.cursor.index = self.value.chars().count();
    }

    fn add(&mut self, c: char) {
        let byte_index = self
            .value
            .char_indices()
            .nth(self.cursor.index)
            .map(|(i, _)| i)
            .unwrap_or(self.value.len());

        self.value.insert(byte_index, c);
        self.cursor_right();
    }

    fn suppr(&mut self) {
        if self.cursor.index == 0 {
            return;
        }

        let byte_index = self
            .value
            .char_indices()
            .nth(self.cursor.index - 1)
            .map(|(i, _)| i);

        if let Some(byte_index) = byte_index {
            self.value.remove(byte_index);
            self.cursor_left();
        }
    }

    fn delete(&mut self) {
        if self.cursor.index < self.value.chars().count() {
            let byte_index = self
                .value
                .char_indices()
                .nth(self.cursor.index)
                .map(|(i, _)| i)
                .unwrap_or(self.value.len());

            self.value.remove(byte_index);
        }
    }

    fn cursor_left(&mut self) {
        if self.cursor.index > 0 {
            self.cursor.index -= 1;
        }
    }

    fn cursor_right(&mut self) {
        if self.cursor.index < self.value.chars().count() {
            self.cursor.index += 1;
        }
    }

    fn line(&self, visible_count: usize) -> Line<'static> {
        let visible: String = self
            .value
            .chars()
            .skip(self.offset)
            .take(visible_count)
            .collect();

        if !self.is_focused || !self.cursor.is_visible {
            return Line::from(visible);
        }

        let index = self.cursor.index.saturating_sub(self.offset);

        let before: String = visible.chars().take(index).collect();
        let after: String = visible.chars().skip(index + 1).collect();
        let under = visible.chars().nth(index).unwrap_or(' ').to_string();

        Line::from(vec![
            Span::raw(before),
            Span::styled(under, Style::default().add_modifier(Modifier::REVERSED)),
            Span::raw(after),
        ])
    }
}

impl InteractiveComponent for TextInput {
    fn render(&mut self, _state: &AppState, frame: &mut Frame, area: Rect) {
        let text_style = if self.is_focused {
            Style::default().fg(ITEM_COLOR)
        } else {
            dim_style()
        };

        let block = default_block()
            .title(format!(" {} ", self.label.as_str()))
            .style(text_style);

        let visible_count = block.inner(area).width as usize;
        self.clamp_offset(visible_count);

        let paragraph = Paragraph::new(self.line(visible_count)).block(block);
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

        if let CrosstermEvent::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                KeyCode::Char('a' | 'A') if modifiers.contains(KeyModifiers::CONTROL) => {
                    self.cursor.index = 0;
                    EventFlow::Consumed
                }
                KeyCode::Char('e' | 'E') if modifiers.contains(KeyModifiers::CONTROL) => {
                    self.cursor_to_end();
                    EventFlow::Consumed
                }
                KeyCode::Char(c) => {
                    self.add(*c);
                    EventFlow::Consumed
                }
                KeyCode::Delete => {
                    self.delete();
                    EventFlow::Consumed
                }
                KeyCode::Backspace => {
                    self.suppr();
                    EventFlow::Consumed
                }
                KeyCode::Left => {
                    self.cursor_left();
                    EventFlow::Consumed
                }
                KeyCode::Right => {
                    self.cursor_right();
                    EventFlow::Consumed
                }
                _ => EventFlow::Ignored,
            }
        } else {
            EventFlow::Ignored
        }
    }
}

impl Lifecycle for TextInput {
    fn on_tick(&mut self, _state: &mut AppState, _sender: &Sender<ApplicationEvent>) {
        if self.cursor.last_blink_instant.elapsed() > CURSOR_BLINK_DURATION {
            self.cursor.is_visible = !self.cursor.is_visible;
            self.cursor.last_blink_instant = Instant::now();
        }
    }
}
