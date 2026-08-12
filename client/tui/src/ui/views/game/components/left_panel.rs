use crate::data::manifest::NpcType;
use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::{GameFocus, Npc};
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
    pub selected_item_index: usize,
    pub quests_area: Option<Rect>,
    pub selected_quest_index: usize,
}

impl LeftPanelComponent {
    pub fn new() -> Self {
        Self {
            npcs_area: None,
            selected_npc_index: None,
            items_area: None,
            selected_item_index: 0,
            quests_area: None,
            selected_quest_index: 0,
        }
    }
}

fn next_alive_npc_index(npcs: &[Npc], current: Option<usize>, forward: bool) -> Option<usize> {
    let alive_indices: Vec<_> = npcs
        .iter()
        .enumerate()
        .filter(|(_, npc)| npc.is_alive)
        .map(|(i, _)| i)
        .collect();

    if alive_indices.is_empty() {
        return None;
    }

    let current_pos = current.and_then(|idx| alive_indices.iter().position(|&i| i == idx));

    let index = if forward {
        current_pos.map_or(0, |p| {
            if p >= alive_indices.len() - 1 {
                0
            } else {
                p + 1
            }
        })
    } else {
        current_pos.map_or(alive_indices.len() - 1, |p| {
            if p == 0 {
                alive_indices.len() - 1
            } else {
                p - 1
            }
        })
    };

    Some(alive_indices[index])
}

impl Component for LeftPanelComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        // TODO: refactoriser cette fonction (decouper en plus petit bloc)

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
            .room
            .players
            .iter()
            .map(|name| {
                let mut style = Style::default();
                if Some(name) == state.game.player.name.as_ref() {
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
            .room
            .npcs
            .iter()
            .map(|npc| {
                let (mut display_name, npc_type) =
                    match state.game.manifest.npcs.get(npc.id.as_str()) {
                        Some(entry) => (entry.name.clone(), entry.npc_type.clone()),
                        None => (npc.id.clone(), NpcType::Normal),
                    };

                let mut color = match npc_type {
                    NpcType::Enemy => Color::Red,
                    NpcType::QuestGiver => Color::Yellow,
                    NpcType::Dialogue => Color::Blue,
                    NpcType::Normal => Color::White,
                };

                if !npc.is_alive {
                    color = Color::Gray;
                    display_name = format!("(dead) {}", display_name);
                }

                let mut style = Style::default().fg(color);

                if let Some(selected_idx) = self.selected_npc_index {
                    // Find the actual index of the npc_id in the room_npcs list
                    if let Some(idx) = state.game.room.npcs.iter().position(|n| n.id == npc.id) {
                        if idx == selected_idx && npc.is_alive {
                            if state.game.ui.current_focus == GameFocus::NpcList {
                                style = style.add_modifier(Modifier::REVERSED);
                            }
                        }
                    }
                }

                ListItem::new(Span::styled(
                    format!("• {} ({})", display_name, npc.id),
                    style,
                ))
            })
            .collect();
        let mut npcs_block = default_block().title(" Room NPCs ");
        if state.game.ui.current_focus == GameFocus::NpcList {
            npcs_block = npcs_block.border_style(Style::default().fg(Color::Yellow));
        }
        let npcs_list = List::new(npcs_items).block(npcs_block);
        frame.render_widget(npcs_list, chunks[1]);
        self.npcs_area = Some(chunks[1]);

        // 3. Room Items
        let items: Vec<ListItem> = state
            .game
            .room
            .items
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

                if self.selected_item_index == idx
                    && state.game.ui.current_focus == GameFocus::RoomItemsList
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
        if state.game.ui.current_focus == GameFocus::RoomItemsList {
            items_block = items_block.border_style(Style::default().fg(Color::Yellow));
        }
        let items_list = List::new(items).block(items_block);
        frame.render_widget(items_list, chunks[2]);
        self.items_area = Some(chunks[2]);

        // 4. Quests
        let quests_items: Vec<ListItem> = state
            .game
            .player
            .quests
            .iter()
            .enumerate()
            .map(|(idx, q)| {
                let desc = state
                    .game
                    .manifest
                    .quests
                    .get(&q.quest_id)
                    .map(|c| c.description.clone())
                    .unwrap_or_else(|| q.quest_id.clone());
                let is_done = q.status.eq_ignore_ascii_case("completed");
                let mut style = if is_done {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                if self.selected_quest_index == idx
                    && state.game.ui.current_focus == GameFocus::QuestList
                {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                ListItem::new(Span::styled(
                    format!("[{}] {}", q.status.to_uppercase(), desc),
                    style,
                ))
            })
            .collect();
        let mut quests_block = default_block().title(" Quests ");
        if state.game.ui.current_focus == GameFocus::QuestList {
            quests_block = quests_block.border_style(Style::default().fg(Color::Yellow));
        }
        let quests_list = List::new(quests_items).block(quests_block);
        frame.render_widget(quests_list, chunks[3]);
        self.quests_area = Some(chunks[3]);
    }
}

impl Lifecycle for LeftPanelComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if state.game.ui.current_focus == GameFocus::NpcList {
            if let crossterm::event::Event::Key(key) = event {
                let npc_count = state.game.room.npcs.len();

                if npc_count > 0 {
                    match key.code {
                        crossterm::event::KeyCode::Up => {
                            if let Some(idx) = next_alive_npc_index(
                                &state.game.room.npcs,
                                self.selected_npc_index,
                                false,
                            ) {
                                self.selected_npc_index = Some(idx);
                            }
                            return true;
                        }
                        crossterm::event::KeyCode::Down => {
                            if let Some(idx) = next_alive_npc_index(
                                &state.game.room.npcs,
                                self.selected_npc_index,
                                true,
                            ) {
                                self.selected_npc_index = Some(idx);
                            }
                            return true;
                        }
                        crossterm::event::KeyCode::Enter => {
                            if !state.game.ui.is_npc_dialogue_available() {
                                return true;
                            }

                            if let Some(idx) = self.selected_npc_index {
                                if let Some(npc) = state.game.room.npcs.get(idx) {
                                    state.game.ui.active_npc_popup = Some(npc.id.clone());
                                    return true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        } else if state.game.ui.current_focus == GameFocus::RoomItemsList {
            if let crossterm::event::Event::Key(key) = event {
                let item_count = state.game.room.items.len();
                if item_count > 0 {
                    match key.code {
                        crossterm::event::KeyCode::Up => {
                            let current = self.selected_item_index;
                            self.selected_item_index = if current == 0 {
                                item_count - 1
                            } else {
                                current - 1
                            };
                            return true;
                        }
                        crossterm::event::KeyCode::Down => {
                            let current = self.selected_item_index;
                            self.selected_item_index = if current >= item_count - 1 {
                                0
                            } else {
                                current + 1
                            };
                            return true;
                        }
                        crossterm::event::KeyCode::Enter => {
                            if let Some(item_id) =
                                state.game.room.items.get(self.selected_item_index)
                            {
                                state.game.ui.active_item_popup = Some(item_id.clone());
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        } else if state.game.ui.current_focus == GameFocus::QuestList {
            if let crossterm::event::Event::Key(key) = event {
                let quest_count = state.game.player.quests.len();
                if quest_count > 0 {
                    match key.code {
                        crossterm::event::KeyCode::Up => {
                            let current = self.selected_quest_index;
                            self.selected_quest_index = if current == 0 {
                                quest_count - 1
                            } else {
                                current - 1
                            };
                            return true;
                        }
                        crossterm::event::KeyCode::Down => {
                            let current = self.selected_quest_index;
                            self.selected_quest_index = if current >= quest_count - 1 {
                                0
                            } else {
                                current + 1
                            };
                            return true;
                        }
                        crossterm::event::KeyCode::Enter => {
                            if let Some(_quest_id) =
                                state.game.player.quests.get(self.selected_quest_index)
                            {
                                state
                                    .game
                                    .log_action("TODO: overlay for quest selection".to_string());
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
