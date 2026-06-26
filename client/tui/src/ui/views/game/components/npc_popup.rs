use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::Component;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem},
};
use tokio::sync::mpsc;

pub struct NpcActionPopup {
    pub selected_action_index: usize,
}

impl NpcActionPopup {
    pub fn new() -> Self {
        Self {
            selected_action_index: 0,
        }
    }

    fn get_available_actions(&self, state: &AppState, npc_id: &str) -> Vec<String> {
        let mut actions = Vec::new();
        if let Some(npc) = state.game.manifest.npcs.get(npc_id) {
            for action in &npc.actions {
                actions.push(action.to_uppercase());
            }
        }
        actions.push("CANCEL".to_string());
        actions
    }
}

impl Component for NpcActionPopup {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let npc_id = if let Some(id) = &state.ui.active_npc_popup {
            id
        } else {
            return;
        };

        let actions = self.get_available_actions(state, npc_id);

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
            .npcs
            .get(npc_id)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| npc_id.clone());

        let title = format!(" {} ", display_name);

        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = actions
            .iter()
            .enumerate()
            .map(|(i, action)| {
                let mut style = Style::default().fg(Color::White);
                if i == self.selected_action_index {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                ListItem::new(Span::styled(format!(" {}", action), style))
            })
            .collect();

        let list = List::new(items).block(
            crate::ui::theme::overlay_block()
                .title(title)
                .style(Style::default().fg(Color::Yellow)),
        );

        frame.render_widget(list, popup_area);
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &mpsc::Sender<ApplicationEvent>,
    ) -> bool {
        let npc_id = if let Some(id) = state.ui.active_npc_popup.clone() {
            id
        } else {
            return false;
        };

        if let CrosstermEvent::Key(key) = event {
            let actions = self.get_available_actions(state, &npc_id);
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
                    state.ui.active_npc_popup = None;
                    self.selected_action_index = 0;
                    return true;
                }
                KeyCode::Enter => {
                    if let Some(action) = actions.get(self.selected_action_index) {
                        if action != "CANCEL" {
                            let cmd = format!("{} {}", action.to_uppercase(), npc_id);
                            let _ = event_sender.try_send(ApplicationEvent::SendRawCommand(cmd));
                        }
                    }
                    state.ui.active_npc_popup = None;
                    self.selected_action_index = 0;
                    return true;
                }
                _ => {
                    // Block all other keys
                    return true;
                }
            }
        }
        true // Block mouse clicks too while popup is active
    }
}
