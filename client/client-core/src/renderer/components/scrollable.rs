use super::component::Component;
use super::lifecycle::{EventFlow, Lifecycle};
use crate::collections::Step;
use crate::events::ApplicationEvent;
use crate::states::AppState;
use crossterm::event::{KeyCode, KeyEvent};
use mpsc::Sender;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};
use tokio::sync::mpsc;

const PAGE_STEP: u16 = 10;

pub trait ScrollableComponent: Lifecycle {
    fn is_scrollable(&self, _state: &AppState) -> bool {
        true
    }

    fn get_area(&self, _state: &AppState, max_area: Rect) -> Rect {
        max_area
    }

    fn get_block<'a>(&self, state: &AppState) -> Block<'a>;

    fn get_content<'a>(&self, state: &'a AppState, max_width: usize) -> Vec<Line<'a>>;
}

pub struct Scrollable<T: ScrollableComponent> {
    pub inner: T,
    pub scroll_offset: u16,
    pub last_max_scroll: u16,
    area: Option<Rect>,
}

impl<T: ScrollableComponent> Scrollable<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            scroll_offset: u16::MAX,
            last_max_scroll: 0,
            area: None,
        }
    }

    fn scroll(&mut self, step: Step, amount: u16) {
        self.scroll_offset = match step {
            Step::Previous => self
                .scroll_offset
                .saturating_add(amount)
                .min(self.last_max_scroll),
            Step::Next => self.scroll_offset.saturating_sub(amount),
        };
    }
}

impl<T: ScrollableComponent> Component for Scrollable<T> {
    fn drawn_area(&self) -> Option<Rect> {
        self.area
    }

    fn draw(&mut self, state: &AppState, frame: &mut Frame, max_area: Rect) {
        let final_area = self.inner.get_area(state, max_area);
        self.area = Some(final_area);

        let block = self.inner.get_block(state);

        let inner_area = block.inner(final_area);
        let max_width = inner_area.width as usize;

        let lines = self.inner.get_content(state, max_width);

        let content_height = lines.len() as u16;
        let inner_height = inner_area.height;
        let max_scroll = content_height.saturating_sub(inner_height);

        self.last_max_scroll = max_scroll;
        self.scroll_offset = self.scroll_offset.min(max_scroll);

        let actual_scroll = max_scroll.saturating_sub(self.scroll_offset);
        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((actual_scroll, 0));

        frame.render_widget(Clear, final_area);
        frame.render_widget(paragraph, final_area);
    }
}

impl<T: ScrollableComponent> Lifecycle for Scrollable<T> {
    fn on_key(
        &mut self,
        state: &mut AppState,
        key: &KeyEvent,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if !self.inner.is_scrollable(state) {
            return EventFlow::Ignored;
        }

        let scroll = match key.code {
            KeyCode::Up => Some((Step::Previous, 1)),
            KeyCode::Down => Some((Step::Next, 1)),
            KeyCode::PageUp => Some((Step::Previous, PAGE_STEP)),
            KeyCode::PageDown => Some((Step::Next, PAGE_STEP)),
            _ => None,
        };

        if let Some((step, amount)) = scroll {
            self.scroll(step, amount);
            return EventFlow::Consumed;
        }

        self.inner.on_key(state, key, sender)
    }

    fn on_click(
        &mut self,
        state: &mut AppState,
        column: u16,
        row: u16,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if !self.inner.is_scrollable(state) || !self.hit(column, row) {
            return EventFlow::Ignored;
        }

        self.inner.on_click(state, column, row, sender)
    }

    fn on_scroll(&mut self, state: &mut AppState, step: Step, column: u16, row: u16) -> EventFlow {
        if !self.inner.is_scrollable(state) || !self.hit(column, row) {
            return EventFlow::Ignored;
        }

        self.scroll(step, 1);

        EventFlow::Consumed
    }

    fn on_tick(&mut self, state: &mut AppState, sender: &Sender<ApplicationEvent>) {
        self.inner.on_tick(state, sender);
    }
}
