use crate::collections::Step;
use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::NpcActionsState;
use crate::ui::components::{Component, EventFlow, Lifecycle, is_mouse_in_rect};
use crate::ui::layout::centered_rect;
use crate::ui::theme::{popup_block, selection_style};
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
pub struct NpcActionsPopup {
    area: Option<Rect>,
    list_area: Option<Rect>,
}

impl NpcActionsPopup {
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
        npc_id: &str,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        let command = state
            .game
            .overlays
            .get::<NpcActionsState>()
            .and_then(|overlay| overlay.selected_command());

        if let Some(command) = command {
            let raw_command = format!("{} {}", command, npc_id);
            let _ = event_sender.try_send(ApplicationEvent::SendRawCommand(raw_command));
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
        let Some(overlay) = state.game.overlays.get::<NpcActionsState>() else {
            return;
        };

        let Some(npc) = state.game.find_npc(&overlay.npc_id) else {
            return;
        };

        let title = format!(" {} ", npc.name);
        let popup_area = centered_rect(area, POPUP_WIDTH, overlay.actions.len() as u16 + 2);

        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = overlay
            .actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let style = selection_style(Color::Reset, overlay.actions.is_selected(index));

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
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        let Some(overlay) = state.game.overlays.get::<NpcActionsState>() else {
            return EventFlow::Ignored;
        };

        let npc_id = overlay.npc_id.clone();

        if state.game.find_npc(&npc_id).is_none() {
            state.game.overlays.close::<NpcActionsState>();
            return EventFlow::Consumed;
        }

        match event {
            CrosstermEvent::Key(key) => match key.code {
                KeyCode::Up | KeyCode::Down => {
                    let step = if key.code == KeyCode::Up {
                        Step::Previous
                    } else {
                        Step::Next
                    };

                    if let Some(overlay) = state.game.overlays.get_mut::<NpcActionsState>() {
                        overlay.actions.move_selection(step);
                    }

                    EventFlow::Consumed
                }
                KeyCode::Esc => {
                    state.game.close_top_overlay();
                    EventFlow::Consumed
                }
                KeyCode::Enter => self.activate(state, &npc_id, event_sender),
                _ => EventFlow::Ignored,
            },
            CrosstermEvent::Mouse(mouse)
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) =>
            {
                let Some(index) = self.hit(mouse.column, mouse.row) else {
                    return EventFlow::Ignored;
                };

                if let Some(overlay) = state.game.overlays.get_mut::<NpcActionsState>() {
                    overlay.actions.select_index(index);
                }

                self.activate(state, &npc_id, event_sender)
            }
            _ => EventFlow::Ignored,
        }
    }
}
