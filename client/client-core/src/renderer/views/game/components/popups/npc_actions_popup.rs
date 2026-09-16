use crate::collections::Step;
use crate::events::{ApplicationEvent, SendEvent};
use crate::renderer::components::{Component, EventFlow, Lifecycle, hit_row};
use crate::renderer::layout::centered_rect;
use crate::renderer::theme::{popup_block, selection_style};
use crate::states::AppState;
use crate::states::game::NpcActionsState;
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
pub struct NpcActionsPopup {
    area: Option<Rect>,
    list_area: Option<Rect>,
}

impl NpcActionsPopup {
    pub fn new() -> Self {
        Self::default()
    }

    fn hit_action(&self, column: u16, row: u16) -> Option<usize> {
        hit_row(self.list_area, column, row)
    }

    fn close_stale_overlay(&self, state: &mut AppState) -> Option<EventFlow> {
        let Some(npc_actions_state) = state.game.overlays.get::<NpcActionsState>() else {
            return Some(EventFlow::Ignored);
        };

        let npc_id = npc_actions_state.npc_id.clone();

        if state.game.find_npc(&npc_id).is_some() {
            return None;
        }

        state.game.overlays.close::<NpcActionsState>();

        Some(EventFlow::Consumed)
    }

    fn activate(&self, state: &mut AppState, event_sender: &Sender<ApplicationEvent>) -> EventFlow {
        let request = state
            .game
            .overlays
            .get::<NpcActionsState>()
            .and_then(|npc_actions_state| npc_actions_state.selected_request());

        if let Some(request) = request {
            let _ = event_sender.try_send(ApplicationEvent::Send(SendEvent::ApiRequest(request)));
        }

        state.game.close_top_overlay();
        EventFlow::Consumed
    }
}

impl Component for NpcActionsPopup {
    fn drawn_area(&self) -> Option<Rect> {
        self.area
    }

    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let Some(npc_actions_state) = state.game.overlays.get::<NpcActionsState>() else {
            return;
        };

        let Some(npc) = state.game.find_npc(&npc_actions_state.npc_id) else {
            return;
        };

        let title = format!(" {} ", npc.name);
        let popup_area = centered_rect(
            area,
            POPUP_WIDTH,
            npc_actions_state.actions.len() as u16 + 2,
        );

        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = npc_actions_state
            .actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let style =
                    selection_style(Color::Reset, npc_actions_state.actions.is_selected(index));

                ListItem::new(Span::styled(format!(" {}", action.label()), style))
            })
            .collect();

        let block = popup_block(title);
        self.area = Some(popup_area);
        self.list_area = Some(block.inner(popup_area));

        let list = List::new(items).block(block);

        frame.render_widget(list, popup_area);
    }
}

impl Lifecycle for NpcActionsPopup {
    fn on_key(
        &mut self,
        state: &mut AppState,
        key: &KeyEvent,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if let Some(flow) = self.close_stale_overlay(state) {
            return flow;
        }

        match key.code {
            KeyCode::Up | KeyCode::Down => {
                let step = if key.code == KeyCode::Up {
                    Step::Previous
                } else {
                    Step::Next
                };

                if let Some(npc_actions_state) = state.game.overlays.get_mut::<NpcActionsState>() {
                    npc_actions_state.actions.move_selection(step);
                }

                EventFlow::Consumed
            }
            KeyCode::Esc => {
                state.game.close_top_overlay();
                EventFlow::Consumed
            }
            KeyCode::Enter => self.activate(state, sender),
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
        if let Some(flow) = self.close_stale_overlay(state) {
            return flow;
        }

        let Some(index) = self.hit_action(column, row) else {
            return EventFlow::Ignored;
        };

        if let Some(npc_actions_state) = state.game.overlays.get_mut::<NpcActionsState>() {
            npc_actions_state.actions.select_index(index);
        }

        self.activate(state, sender)
    }
}
