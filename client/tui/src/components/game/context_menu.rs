use crate::commands::handle_command;
use crate::components::Component;
use crate::events::AppEvent;
use crate::state::AppState;
use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
};

pub struct ContextMenuComponent;

impl ContextMenuComponent {
    pub fn get_available_actions(&self, _entity_name: &str) -> Vec<&'static str> {
        vec!["Talk", "Attack"]
    }
}

#[async_trait::async_trait]
impl Component for ContextMenuComponent {
    async fn handle_event(
        &mut self,
        state: &mut AppState,
        event: &Event,
        tx: &tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => {
                    state.ui.context_menu_idx = state.ui.context_menu_idx.saturating_sub(1);
                }
                KeyCode::Down => {
                    let entity_name = state
                        .game
                        .npcs_in_room
                        .get(state.ui.selected_entity_idx.unwrap_or(0))
                        .cloned()
                        .unwrap_or_default();
                    let max_idx = self
                        .get_available_actions(&entity_name)
                        .len()
                        .saturating_sub(1);
                    if state.ui.context_menu_idx < max_idx {
                        state.ui.context_menu_idx += 1;
                    }
                }
                KeyCode::Enter => {
                    // Execute context action
                    let entity_name = state
                        .game
                        .npcs_in_room
                        .get(state.ui.selected_entity_idx.unwrap_or(0))
                        .cloned()
                        .unwrap_or_default();
                    let actions = self.get_available_actions(&entity_name);
                    if let Some(action) = actions.get(state.ui.context_menu_idx) {
                        let cmd =
                            format!("{} {}", action.to_lowercase(), entity_name.to_lowercase());
                        handle_command(state, cmd, tx.clone());
                    }
                    state.ui.context_menu_open = false;
                }
                KeyCode::Esc => {
                    state.ui.context_menu_open = false;
                }
                _ => {}
            }
        }
    }

    fn draw(&mut self, state: &mut AppState, f: &mut Frame, area: Rect) {
        if !state.ui.context_menu_open {
            return;
        }

        if let Some(idx) = state.ui.selected_entity_idx {
            if let Some(entity_name) = state.game.npcs_in_room.get(idx) {
                let actions = self.get_available_actions(entity_name);
                let items: Vec<ListItem> = actions
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        if i == state.ui.context_menu_idx {
                            ListItem::new(format!("> {}", a))
                                .style(Style::default().fg(Color::Yellow))
                        } else {
                            ListItem::new(format!("  {}", a))
                        }
                    })
                    .collect();
                let menu = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(" Action "));

                // Position it relative to the scene inner area (area is the inner area passed by mod.rs)
                let menu_rect = Rect {
                    x: area.x + 2,
                    y: area.y + 2,
                    width: 15,
                    height: actions.len() as u16 + 2,
                };
                f.render_widget(Clear, menu_rect);
                f.render_widget(menu, menu_rect);
            }
        }
    }
}
