use crate::collections::Step;
use crate::events::{ApplicationEvent, SendEvent};
use crate::renderer::components::{Component, EventFlow, Lifecycle, hit_row};
use crate::renderer::layout::centered_rect;
use crate::renderer::theme::{popup_block, selection_style};
use crate::states::AppState;
use crate::states::game::{ItemActionsState, ItemDetailState, Overlay};
use client_api::ApiRequest;
use client_api::commands::{DropCommand, TakeCommand, UseCommand};
use crossterm::event::{KeyCode, KeyEvent};
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

    fn hit_action(&self, column: u16, row: u16) -> Option<usize> {
        hit_row(self.list_area, column, row)
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
            .and_then(|item_actions_state| item_actions_state.actions.selected().cloned());

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
            Some(ItemActionsState::TAKE) => {
                let _ = event_sender.try_send(ApplicationEvent::Send(SendEvent::ApiRequest(
                    ApiRequest::Take(TakeCommand {
                        item_identifier: item_id.to_string(),
                    }),
                )));
            }
            Some(ItemActionsState::USE) => {
                let _ = event_sender.try_send(ApplicationEvent::Send(SendEvent::ApiRequest(
                    ApiRequest::Use(UseCommand {
                        item_identifier: item_id.to_string(),
                    }),
                )));
            }
            Some(ItemActionsState::DROP) => {
                let _ = event_sender.try_send(ApplicationEvent::Send(SendEvent::ApiRequest(
                    ApiRequest::Drop(DropCommand {
                        item_identifier: item_id.to_string(),
                    }),
                )));
            }
            Some(ItemActionsState::CANCEL) | None => {}
            Some(_) => {}
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
        let Some(item_actions_state) = state.game.overlays.get::<ItemActionsState>() else {
            return;
        };

        let Some(item) = state.game.find_item(&item_actions_state.item_id) else {
            return;
        };

        let title = format!(" {} ", item.name);
        let popup_area = centered_rect(
            area,
            POPUP_WIDTH,
            item_actions_state.actions.len() as u16 + 2,
        );

        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = item_actions_state
            .actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let style =
                    selection_style(Color::Reset, item_actions_state.actions.is_selected(index));

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
    fn on_key(
        &mut self,
        state: &mut AppState,
        key: &KeyEvent,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        let Some(item_actions_state) = state.game.overlays.get::<ItemActionsState>() else {
            return EventFlow::Ignored;
        };

        let item_id = item_actions_state.item_id.clone();

        match key.code {
            KeyCode::Up | KeyCode::Down => {
                let step = if key.code == KeyCode::Up {
                    Step::Previous
                } else {
                    Step::Next
                };

                if let Some(item_actions_state) = state.game.overlays.get_mut::<ItemActionsState>()
                {
                    item_actions_state.actions.move_selection(step);
                }

                EventFlow::Consumed
            }
            KeyCode::Esc => {
                state.game.close_top_overlay();
                EventFlow::Consumed
            }
            KeyCode::Enter => self.activate(state, &item_id, sender),
            _ => EventFlow::Ignored,
        }
    }

    fn on_click(
        &mut self,
        state: &mut AppState,
        column: u16,
        row: u16,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        let Some(item_actions_state) = state.game.overlays.get::<ItemActionsState>() else {
            return EventFlow::Ignored;
        };

        let item_id = item_actions_state.item_id.clone();

        let Some(index) = self.hit_action(column, row) else {
            return EventFlow::Ignored;
        };

        if let Some(item_actions_state) = state.game.overlays.get_mut::<ItemActionsState>() {
            item_actions_state.actions.select_index(index);
        }

        self.activate(state, &item_id, sender)
    }
}
