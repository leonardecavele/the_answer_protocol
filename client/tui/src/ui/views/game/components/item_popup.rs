use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::theme::overlay_block;
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

pub struct ItemPopupComponent {
    pub selected_action_index: usize,
}

impl ItemPopupComponent {
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

impl Component for ItemPopupComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let item_id = if let Some(id) = &state.game.ui.active_item_popup {
            id
        } else {
            return;
        };

        let actions = self.get_actions(state, item_id);

        let width = 30;
        let height = actions.len() as u16 + 2; // +2 for borders
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let popup_area = Rect {
            x,
            y,
            width,
            height,
        };

        let display_name = state
            .game
            .manifest
            .items
            .get(item_id)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| item_id.clone());

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

impl Lifecycle for ItemPopupComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        let item_id = if let Some(id) = state.game.ui.active_item_popup.clone() {
            id
        } else {
            return false;
        };

        if let CrosstermEvent::Key(key) = event {
            let actions = self.get_actions(state, &item_id);
            let count = actions.len();

            match key.code {
                KeyCode::Up => {
                    self.selected_action_index = if self.selected_action_index == 0 {
                        count.saturating_sub(1)
                    } else {
                        self.selected_action_index - 1
                    };
                    return true;
                }
                KeyCode::Down => {
                    self.selected_action_index =
                        if self.selected_action_index >= count.saturating_sub(1) {
                            0
                        } else {
                            self.selected_action_index + 1
                        };
                    return true;
                }
                KeyCode::Esc => {
                    state.game.ui.active_item_popup = None;
                    self.selected_action_index = 0;
                    return true;
                }
                KeyCode::Enter => {
                    if let Some(act) = actions.get(self.selected_action_index) {
                        match act.as_str() {
                            "VIEW" => {
                                state.game.ui.active_item_view_popup = Some(item_id.clone());
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

                    state.game.ui.active_item_popup = None;
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
