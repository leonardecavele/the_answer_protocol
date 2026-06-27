use crate::data::manifest::NpcType;
use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::ui::GameFocus;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::theme::default_block;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{List, ListItem},
};
use tokio::sync::mpsc::Sender;

pub struct LeftPanelComponent {
    pub npcs_area: Option<Rect>,
    pub selected_npc_index: Option<usize>,
    pub items_area: Option<Rect>,
}

impl LeftPanelComponent {
    pub fn new() -> Self {
        Self {
            npcs_area: None,
            selected_npc_index: None,
            items_area: None,
        }
    }
}

impl Component for LeftPanelComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
            ])
            .split(area);

        // 1. Room Players
        let players_items: Vec<ListItem> = state
            .game
            .room_players
            .iter()
            .map(|name| {
                let mut style = Style::default();
                if Some(name) == state.game.player_name.as_ref() {
                    style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                }
                ListItem::new(Span::styled(format!("• {}", name), style))
            })
            .collect();
        let players_list = List::new(players_items).block(default_block().title(" Room Players "));
        frame.render_widget(players_list, chunks[0]);

        // 2. Room NPCs
        let npcs_items: Vec<ListItem> = state
            .game
            .room_npcs
            .iter()
            .map(|npc_id| {
                let (display_name, npc_type) = match state.game.manifest.npcs.get(npc_id) {
                    Some(entry) => (entry.name.clone(), entry.npc_type.clone()),
                    None => (npc_id.clone(), NpcType::Normal),
                };

                let color = match npc_type {
                    NpcType::Enemy => Color::Red,
                    NpcType::QuestGiver => Color::Yellow,
                    NpcType::Dialogue => Color::Blue,
                    NpcType::Normal => Color::White,
                };

                let mut style = Style::default().fg(color);

                if let Some(selected_idx) = self.selected_npc_index {
                    // Find the actual index of the npc_id in the room_npcs list
                    if let Some(idx) = state.game.room_npcs.iter().position(|id| id == npc_id) {
                        if idx == selected_idx {
                            if state.ui.current_focus == GameFocus::NpcList {
                                style = style.add_modifier(Modifier::REVERSED);
                            }
                        }
                    }
                }

                ListItem::new(Span::styled(
                    format!("• {} ({})", display_name, npc_id),
                    style,
                ))
            })
            .collect();
        let mut npcs_block = default_block().title(" Room NPCs ");
        if state.ui.current_focus == GameFocus::NpcList {
            npcs_block = npcs_block.border_style(Style::default().fg(Color::Yellow));
        }
        let npcs_list = List::new(npcs_items).block(npcs_block);
        frame.render_widget(npcs_list, chunks[1]);
        self.npcs_area = Some(chunks[1]);

        // 3. Room Items
        let items: Vec<ListItem> = state
            .game
            .current_room_items
            .iter()
            .enumerate()
            .map(|(idx, item_id)| {
                let display_name = state
                    .game
                    .manifest
                    .items
                    .get(item_id)
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| item_id.clone());
                let mut style = Style::default().fg(Color::Cyan);

                if state.game.room_item_cursor == idx
                    && state.ui.current_focus == GameFocus::RoomItemsList
                {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                ListItem::new(Span::styled(
                    format!("• {} ({})", display_name, item_id),
                    style,
                ))
            })
            .collect();
        let mut items_block = default_block().title(" Room Items ");
        if state.ui.current_focus == GameFocus::RoomItemsList {
            items_block = items_block.border_style(Style::default().fg(Color::Yellow));
        }
        let items_list = List::new(items).block(items_block);
        frame.render_widget(items_list, chunks[2]);
        self.items_area = Some(chunks[2]);

        // 4. Quests
        let quests_items: Vec<ListItem> = state
            .game
            .quests
            .iter()
            .map(|q| {
                let desc = state
                    .game
                    .manifest
                    .quests
                    .get(&q.quest_id)
                    .map(|c| c.description.clone())
                    .unwrap_or_else(|| q.quest_id.clone());
                let is_done = q.status.eq_ignore_ascii_case("completed");
                let style = if is_done {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Yellow)
                };
                ListItem::new(Span::styled(
                    format!("[{}] {}", q.status.to_uppercase(), desc),
                    style,
                ))
            })
            .collect();
        let quests_block = default_block().title(" Quests ");
        let quests_list = List::new(quests_items).block(quests_block);
        frame.render_widget(quests_list, chunks[3]);
    }
}

impl Lifecycle for LeftPanelComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if state.ui.current_focus == GameFocus::NpcList {
            if let crossterm::event::Event::Key(key) = event {
                let npc_count = state.game.room_npcs.len();
                if npc_count > 0 {
                    match key.code {
                        crossterm::event::KeyCode::Up => {
                            let current = self.selected_npc_index.unwrap_or(0);
                            self.selected_npc_index = Some(if current == 0 {
                                npc_count - 1
                            } else {
                                current - 1
                            });
                            return true;
                        }
                        crossterm::event::KeyCode::Down => {
                            let current = self.selected_npc_index.unwrap_or(npc_count - 1);
                            self.selected_npc_index = Some(if current >= npc_count - 1 {
                                0
                            } else {
                                current + 1
                            });
                            return true;
                        }
                        crossterm::event::KeyCode::Enter => {
                            if let Some(idx) = self.selected_npc_index {
                                if let Some(npc_id) = state.game.room_npcs.get(idx) {
                                    state.ui.active_npc_popup = Some(npc_id.clone());
                                    return true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        } else if state.ui.current_focus == GameFocus::RoomItemsList {
            if let crossterm::event::Event::Key(key) = event {
                let item_count = state.game.current_room_items.len();
                if item_count > 0 {
                    match key.code {
                        crossterm::event::KeyCode::Up => {
                            let current = state.game.room_item_cursor;
                            state.game.room_item_cursor = if current == 0 {
                                item_count - 1
                            } else {
                                current - 1
                            };
                            return true;
                        }
                        crossterm::event::KeyCode::Down => {
                            let current = state.game.room_item_cursor;
                            state.game.room_item_cursor = if current >= item_count - 1 {
                                0
                            } else {
                                current + 1
                            };
                            return true;
                        }
                        crossterm::event::KeyCode::Enter => {
                            if let Some(item_id) = state
                                .game
                                .current_room_items
                                .get(state.game.room_item_cursor)
                            {
                                state.ui.active_item_popup = Some(item_id.clone());
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        false
    }
}
