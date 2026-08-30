use crate::collections::Step;
use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::NpcActionsState;
use crate::ui::components::{Component, EventFlow, Lifecycle};
use crate::ui::layout::centered_rect;
use crate::ui::theme::{popup_block, selection_style};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
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
pub struct NpcActionsPopup;

impl NpcActionsPopup {
    pub fn new() -> Self {
        Self
    }
}

impl Component for NpcActionsPopup {
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

                ListItem::new(Span::styled(format!(" {}", action), style))
            })
            .collect();

        let list = List::new(items).block(popup_block(title));

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
        let CrosstermEvent::Key(key) = event else {
            return EventFlow::Ignored;
        };

        let Some(overlay) = state.game.overlays.get::<NpcActionsState>() else {
            return EventFlow::Ignored;
        };

        let npc_id = overlay.npc_id.clone();
        let command = overlay.selected_command().map(str::to_string);

        if state.game.find_npc(&npc_id).is_none() {
            state.game.overlays.close::<NpcActionsState>();
            return EventFlow::Consumed;
        }

        match key.code {
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
                state.game.overlays.close_top();
                EventFlow::Consumed
            }
            KeyCode::Enter => {
                if let Some(command) = command {
                    let raw_command = format!("{} {}", command, npc_id);
                    let _ = event_sender.try_send(ApplicationEvent::SendRawCommand(raw_command));
                }

                state.game.overlays.close_top();
                EventFlow::Consumed
            }
            _ => EventFlow::Ignored,
        }
    }
}
