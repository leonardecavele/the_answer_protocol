use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::Component;
use crossterm::event::{Event as CrosstermEvent, MouseEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use tokio::sync::mpsc;

pub trait InteractiveComponent {
    fn render(&mut self, state: &AppState, frame: &mut Frame, area: Rect);

    fn handle_terminal_event(
        &mut self,
        _state: &mut AppState,
        _event: &CrosstermEvent,
        _sender: &mpsc::Sender<ApplicationEvent>,
        _is_hovered: bool,
    ) -> bool {
        false
    }
}

pub struct Interactive<T: InteractiveComponent> {
    pub inner: T,
    pub last_area: Option<Rect>,
}

impl<T: InteractiveComponent> Interactive<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            last_area: None,
        }
    }

    pub fn is_mouse_over(&self, col: u16, row: u16) -> bool {
        if let Some(area) = self.last_area {
            crate::ui::components::is_mouse_in_rect(col, row, area)
        } else {
            false
        }
    }
}

impl<T: InteractiveComponent> Component for Interactive<T> {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.last_area = Some(area);
        self.inner.render(state, frame, area);
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        sender: &mpsc::Sender<ApplicationEvent>,
    ) -> bool {
        let is_hovered = match event {
            CrosstermEvent::Mouse(MouseEvent { column, row, .. }) => {
                self.is_mouse_over(*column, *row)
            }
            _ => false,
        };
        self.inner
            .handle_terminal_event(state, event, sender, is_hovered)
    }
}
