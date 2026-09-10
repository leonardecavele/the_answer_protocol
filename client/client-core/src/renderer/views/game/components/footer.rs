use crate::collections::Step;
use crate::events::{ApplicationEvent, SendEvent};
use crate::renderer::components::{
    Component, EventFlow, Interactive, Lifecycle, TextInput, is_mouse_in_rect,
};
use crate::renderer::theme::{ERROR_COLOR, default_block};
use crate::states::AppState;
use crate::states::game::GameFocus;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Paragraph;
use std::collections::VecDeque;
use tokio::sync::mpsc::Sender;

const LAG_LABEL: &str = "LAG";
const LAG_WIDTH: u16 = 9;
const HISTORY_CAPACITY: usize = 10;

pub enum FooterHit {
    CommandInput,
    None,
}

#[derive(Default)]
pub struct Footer {
    pub input: Interactive<TextInput>,
    tmp_value: Option<String>,
    history: VecDeque<String>,
    history_index: usize,
    area: Option<Rect>,
}

impl Footer {
    pub fn new() -> Self {
        let mut input = Interactive::new(TextInput::new("Command"));
        input.inner.is_focused = true;
        Self {
            input,
            tmp_value: None,
            history: VecDeque::new(),
            history_index: 0,
            area: None,
        }
    }

    pub fn hit(&self, column: u16, row: u16) -> FooterHit {
        if let Some(area) = self.area
            && is_mouse_in_rect(column, row, area)
        {
            return FooterHit::CommandInput;
        }

        FooterHit::None
    }

    fn set_value(&mut self, value: String) {
        self.input.inner.value = value;
        self.input.inner.cursor_to_end();
    }

    fn push_history(&mut self, command: String) {
        self.history.push_back(command);

        if self.history.len() > HISTORY_CAPACITY {
            self.history.pop_front();
        }

        self.history_index = self.history.len();
        self.tmp_value = None;
    }

    fn move_history(&mut self, step: Step) {
        match step {
            Step::Previous => {
                if self.history_index == 0 {
                    return;
                }

                if self.history_index == self.history.len() {
                    self.tmp_value = Some(self.input.inner.value.clone());
                }

                self.history_index -= 1;
                self.set_value(self.history[self.history_index].clone());
            }
            Step::Next => {
                if self.history_index == self.history.len() {
                    return;
                }

                self.history_index += 1;

                let value = match self.history.get(self.history_index) {
                    Some(command) => command.clone(),
                    None => self.tmp_value.take().unwrap_or_default(),
                };

                self.set_value(value);
            }
        }
    }

    fn draw_lag(frame: &mut Frame, area: Rect) {
        let style = Style::default()
            .fg(ERROR_COLOR)
            .add_modifier(Modifier::BOLD);

        let paragraph = Paragraph::new(LAG_LABEL)
            .alignment(Alignment::Center)
            .style(style)
            .block(default_block());

        frame.render_widget(paragraph, area);
    }
}

impl Component for Footer {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let lag_width = if state.network.has_lag { LAG_WIDTH } else { 0 };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(lag_width)])
            .split(area);

        self.area = Some(chunks[0]);
        self.input.inner.is_focused = state.game.focus() == GameFocus::Input;
        self.input.draw(state, frame, chunks[0]);

        if state.network.has_lag {
            Self::draw_lag(frame, chunks[1]);
        }
    }
}

impl Lifecycle for Footer {
    fn on_tick(&mut self, state: &mut AppState, sender: &Sender<ApplicationEvent>) {
        self.input.on_tick(state, sender);
    }

    fn handle_device_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &tokio::sync::mpsc::Sender<ApplicationEvent>,
    ) -> EventFlow {
        if state.game.focus() != GameFocus::Input {
            return EventFlow::Ignored;
        }

        if let CrosstermEvent::Key(key) = event {
            match key.code {
                KeyCode::Enter => {
                    let command = self.input.inner.value.trim().to_string();

                    if command.is_empty() {
                        state.game.set_focus(GameFocus::RightPanel);
                        return EventFlow::Consumed;
                    }

                    self.set_value(String::new());
                    self.push_history(command.clone());

                    let _ = event_sender
                        .try_send(ApplicationEvent::Send(SendEvent::RawCommand(command)));

                    return EventFlow::Consumed;
                }
                KeyCode::Up => {
                    self.move_history(Step::Previous);
                    return EventFlow::Consumed;
                }
                KeyCode::Down => {
                    self.move_history(Step::Next);
                    return EventFlow::Consumed;
                }
                _ => {}
            }
        }

        self.input.handle_device_event(state, event, event_sender)
    }
}
