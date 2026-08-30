use crate::collections::{Step, move_index};
use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::{Npc, NpcActionsState};
use crate::ui::components::Lifecycle;
use crate::ui::components::component::Component;
use crate::ui::components::lifecycle::EventFlow;
use crate::ui::layout::centered_rect;
use crate::ui::theme::popup_block;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use mpsc::Sender;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Clear, List, ListItem},
};
use tokio::sync::mpsc;

const POPUP_WIDTH: u16 = 30;

pub struct NpcActionsPopup {
    pub selected_action_index: usize,
}

impl Default for NpcActionsPopup {
    fn default() -> Self {
        Self::new()
    }
}

impl NpcActionsPopup {
    pub fn new() -> Self {
        Self {
            selected_action_index: 0,
        }
    }

    fn actions_of(npc: &Npc) -> Vec<String> {
        let mut actions: Vec<String> = npc.actions.iter().map(|a| a.to_uppercase()).collect();
        actions.push("CANCEL".to_string());
        actions
    }
}

impl Component for NpcActionsPopup {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let npc_id = match state.game.overlays.get::<NpcActionsState>() {
            Some(overlay) => overlay.npc_id.as_str(),
            None => return,
        };

        let Some(npc) = state.game.find_npc(npc_id) else {
            return;
        };

        let actions = Self::actions_of(npc);
        let title = format!(" {} ", npc.name);

        let popup_area = centered_rect(area, POPUP_WIDTH, actions.len() as u16 + 2);

        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = actions
            .iter()
            .enumerate()
            .map(|(i, action)| {
                let mut style = Style::default().fg(Color::Reset);
                if i == self.selected_action_index {
                    style = style.add_modifier(Modifier::REVERSED);
                }
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
        let npc_id = match state.game.overlays.get::<NpcActionsState>() {
            Some(overlay) => overlay.npc_id.clone(),
            None => return EventFlow::Ignored,
        };

        if let CrosstermEvent::Key(key) = event {
            let Some(actions) = state.game.find_npc(&npc_id).map(Self::actions_of) else {
                state.game.overlays.close::<NpcActionsState>();
                self.selected_action_index = 0;
                return EventFlow::Consumed;
            };

            let count = actions.len();

            match key.code {
                KeyCode::Up => {
                    self.selected_action_index =
                        move_index(self.selected_action_index, count, Step::Previous);
                    return EventFlow::Consumed;
                }
                KeyCode::Down => {
                    self.selected_action_index =
                        move_index(self.selected_action_index, count, Step::Next);
                    return EventFlow::Consumed;
                }
                KeyCode::Esc => {
                    state.game.overlays.close_top();
                    self.selected_action_index = 0;
                    return EventFlow::Consumed;
                }
                KeyCode::Enter => {
                    if let Some(action) = actions.get(self.selected_action_index)
                        && action != "CANCEL"
                    {
                        let cmd = format!("{} {}", action.to_uppercase(), npc_id);
                        let _ = event_sender.try_send(ApplicationEvent::SendRawCommand(cmd));
                    }
                    state.game.overlays.close_top();
                    self.selected_action_index = 0;
                    return EventFlow::Consumed;
                }
                _ => {
                    return EventFlow::Ignored;
                }
            }
        }

        EventFlow::Ignored
    }
}
