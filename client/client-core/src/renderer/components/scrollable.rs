use super::component::Component;
use super::interactive::is_mouse_in_rect;
use super::lifecycle::{EventFlow, Lifecycle};
use crate::events::ApplicationEvent;
use crate::states::AppState;
use crossterm::event::{Event as CrosstermEvent, KeyCode, MouseEventKind};
use mpsc::Sender;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};
use tokio::sync::mpsc;

pub enum ScrollableHit {
    Box,
    None,
}

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

#[derive(Default)]
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
            scroll_offset: 0,
            last_max_scroll: 0,
            area: None,
        }
    }

    pub fn hit(&self, column: u16, row: u16) -> ScrollableHit {
        if let Some(area) = self.area
            && is_mouse_in_rect(column, row, area)
        {
            return ScrollableHit::Box;
        }

        ScrollableHit::None
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
    fn handle_device_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if !self.inner.is_scrollable(state) {
            return EventFlow::Ignored;
        }

        if let CrosstermEvent::Key(key) = event {
            match key.code {
                KeyCode::Up => {
                    self.scroll_offset = self
                        .scroll_offset
                        .saturating_add(1)
                        .min(self.last_max_scroll);
                    return EventFlow::Consumed;
                }
                KeyCode::Down => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                    return EventFlow::Consumed;
                }
                KeyCode::PageUp => {
                    self.scroll_offset = self
                        .scroll_offset
                        .saturating_add(10)
                        .min(self.last_max_scroll);
                    return EventFlow::Consumed;
                }
                KeyCode::PageDown => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(10);
                    return EventFlow::Consumed;
                }
                _ => {}
            }
        } else if let CrosstermEvent::Mouse(mouse) = event {
            if mouse.kind == MouseEventKind::ScrollUp {
                self.scroll_offset = self
                    .scroll_offset
                    .saturating_add(1)
                    .min(self.last_max_scroll);
                return EventFlow::Consumed;
            } else if mouse.kind == MouseEventKind::ScrollDown {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                return EventFlow::Consumed;
            }
        }

        self.inner.handle_device_event(state, event, sender)
    }

    fn on_tick(&mut self, state: &mut AppState, sender: &Sender<ApplicationEvent>) {
        self.inner.on_tick(state, sender);
    }
}
