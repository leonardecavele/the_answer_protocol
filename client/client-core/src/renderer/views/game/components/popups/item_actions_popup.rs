use crate::collections::Step;
use crate::events::ApplicationEvent;
use crate::renderer::components::{Component, EventFlow, Lifecycle, is_mouse_in_rect};
use crate::renderer::layout::centered_rect;
use crate::renderer::theme::{popup_block, selection_style};
use crate::states::AppState;
use crate::states::game::{ItemActionsState, ItemDetailState, Overlay};
use crossterm::event::{Event as CrosstermEvent, KeyCode, MouseButton, MouseEventKind};
use mpsc::Sender;
use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    text::Span,
    widgets::{Clear, List, ListItem},
};
use tokio::sync::mpsc;

const POPUP_WIDTH: u16 = 30;

#[derive(Default)]
pub struct ItemActionsPopup {
    area: Option<Rect>,
    list_area: Option<Rect>,
}

impl ItemActionsPopup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hit(&self, column: u16, row: u16) -> Option<usize> {
        let area = self.list_area?;

        if !is_mouse_in_rect(column, row, area) {
            return None;
        }

        Some(row.saturating_sub(area.y) as usize)
    }

    fn activate(
        &self,
        state: &mut AppState,
        item_id: &str,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        let selected = state
            .game
            .overlays
            .get::<ItemActionsState>()
            .and_then(|overlay| overlay.actions.selected().cloned());

        match selected.as_deref() {
            Some(ItemActionsState::VIEW) => {
                state
                    .game
                    .overlays
                    .open(Overlay::ItemDetail(ItemDetailState::new(
                        item_id.to_string(),
                    )));
                return EventFlow::Consumed;
            }
            Some(ItemActionsState::CANCEL) | None => {}
            Some(action) => {
                let raw_command = format!("{} {}", action, item_id);
                let _ = event_sender.try_send(ApplicationEvent::SendRawCommand(raw_command));
            }
        }

        state.game.close_top_overlay();
        EventFlow::Consumed
    }
}

impl Component for ItemActionsPopup {
    fn drawn_area(&self) -> Option<Rect> {
        self.area
    }

    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let Some(overlay) = state.game.overlays.get::<ItemActionsState>() else {
            return;
        };

        let Some(item) = state.game.find_item(&overlay.item_id) else {
            return;
        };

        let title = format!(" {} ", item.name);
        let popup_area = centered_rect(area, POPUP_WIDTH, overlay.actions.len() as u16 + 2);

        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = overlay
            .actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let style = selection_style(Color::Reset, overlay.actions.is_selected(index));

                ListItem::new(Span::styled(format!(" {}", action), style))
            })
            .collect();

        let block = popup_block(title);
        self.area = Some(popup_area);
        self.list_area = Some(block.inner(popup_area));

        let list = List::new(items).block(block);

        frame.render_widget(list, popup_area);
    }
}

impl Lifecycle for ItemActionsPopup {
    fn handle_device_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        let Some(overlay) = state.game.overlays.get::<ItemActionsState>() else {
            return EventFlow::Ignored;
        };

        let item_id = overlay.item_id.clone();

        match event {
            CrosstermEvent::Key(key) => match key.code {
                KeyCode::Up | KeyCode::Down => {
                    let step = if key.code == KeyCode::Up {
                        Step::Previous
                    } else {
                        Step::Next
                    };

                    if let Some(overlay) = state.game.overlays.get_mut::<ItemActionsState>() {
                        overlay.actions.move_selection(step);
                    }

                    EventFlow::Consumed
                }
                KeyCode::Esc => {
                    state.game.close_top_overlay();
                    EventFlow::Consumed
                }
                KeyCode::Enter => self.activate(state, &item_id, event_sender),
                _ => EventFlow::Ignored,
            },
            CrosstermEvent::Mouse(mouse)
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) =>
            {
                let Some(index) = self.hit(mouse.column, mouse.row) else {
                    return EventFlow::Ignored;
                };

                if let Some(overlay) = state.game.overlays.get_mut::<ItemActionsState>() {
                    overlay.actions.select_index(index);
                }

                self.activate(state, &item_id, event_sender)
            }
            _ => EventFlow::Ignored,
        }
    }
}
