use crate::collections::{Step, move_index};
use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::{Overlay, OverlayKind};
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::theme::overlay_block;
use crate::ui::utils::centered_rect;
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

pub struct ItemActionsPopup {
    pub selected_action_index: usize,
}

impl ItemActionsPopup {
    pub fn new() -> Self {
        Self {
            selected_action_index: 0,
        }
    }

    fn get_actions(&self, state: &AppState, item_id: &str) -> Vec<String> {
        let mut actions = Vec::new();
        if state.game.room.items.contains(&item_id.to_string()) {
            actions.push("TAKE".to_string());
        }
        if state.game.player.inventory.contains(&item_id.to_string()) {
            actions.push("DROP".to_string());
        }
        actions.push("VIEW".to_string());
        actions.push("CANCEL".to_string());
        actions
    }
}

impl Component for ItemActionsPopup {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let item_id = match state.game.overlays.target_of(OverlayKind::ItemActions) {
            Some(id) => id,
            None => return,
        };

        let actions = self.get_actions(state, item_id);

        let popup_area = centered_rect(area, POPUP_WIDTH, actions.len() as u16 + 2);

        let display_name = state
            .game
            .manifest
            .items
            .get(item_id)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| item_id.to_string());

        let title = format!(" {} ", display_name);

        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = actions
            .iter()
            .enumerate()
            .map(|(i, act)| {
                let mut style = Style::default().fg(Color::White);
                if i == self.selected_action_index {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                ListItem::new(Span::styled(format!(" {}", act), style))
            })
            .collect();

        let list = List::new(items).block(
            overlay_block()
                .title(title)
                .style(Style::default().fg(Color::Yellow)),
        );

        frame.render_widget(list, popup_area);
    }
}

impl Lifecycle for ItemActionsPopup {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        let item_id = match state.game.overlays.target_of(OverlayKind::ItemActions) {
            Some(id) => id.to_string(),
            None => return false,
        };

        if let CrosstermEvent::Key(key) = event {
            let actions = self.get_actions(state, &item_id);
            let count = actions.len();

            match key.code {
                KeyCode::Up => {
                    self.selected_action_index =
                        move_index(self.selected_action_index, count, Step::Previous);
                    return true;
                }
                KeyCode::Down => {
                    self.selected_action_index =
                        move_index(self.selected_action_index, count, Step::Next);
                    return true;
                }
                KeyCode::Esc => {
                    state.game.overlays.close_top();
                    self.selected_action_index = 0;
                    return true;
                }
                KeyCode::Enter => {
                    if let Some(act) = actions.get(self.selected_action_index) {
                        match act.as_str() {
                            "VIEW" => {
                                state.game.overlays.open(Overlay::ItemDetail {
                                    item_id: item_id.clone(),
                                });
                                return true;
                            }
                            "CANCEL" => {}
                            _ => {
                                let cmd = format!("{} {}", act.to_uppercase(), item_id);
                                let _ =
                                    event_sender.try_send(ApplicationEvent::SendRawCommand(cmd));
                            }
                        }
                    }

                    state.game.overlays.close_top();
                    self.selected_action_index = 0;
                    return true;
                }
                _ => {
                    return true;
                }
            }
        }
        true
    }
}
