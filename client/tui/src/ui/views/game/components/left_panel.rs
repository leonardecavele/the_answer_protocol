use crate::states::app::AppState;
use crate::ui::components::Component;
use crate::data::manifest::NpcType;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem},
};

pub struct LeftPanelComponent {
    pub npcs_area: Option<Rect>,
    pub selected_npc_index: Option<usize>,
}

impl LeftPanelComponent {
    pub fn new() -> Self {
        Self { npcs_area: None, selected_npc_index: None }
    }
}

impl Component for LeftPanelComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
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
        let players_list = List::new(players_items)
            .block(Block::default().borders(Borders::ALL).title(" Room Players "));
        frame.render_widget(players_list, chunks[0]);

        // 2. Room NPCs
        let npcs_items: Vec<ListItem> = state
            .game
            .room_npcs
            .iter()
            .map(|npc_id| {
                let (display_name, npc_type) =
                    match state.game.manifest.npcs.get(npc_id) {
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
                            if state.ui.current_focus == crate::states::ui::GameFocus::NpcList {
                                style = style.add_modifier(Modifier::REVERSED);
                            }
                        }
                    }
                }

                ListItem::new(Span::styled(
                    format!("• {}", display_name),
                    style,
                ))
            })
            .collect();
        let mut npcs_block = Block::default().borders(Borders::ALL).title(" Room NPCs ");
        if state.ui.current_focus == crate::states::ui::GameFocus::NpcList {
            npcs_block = npcs_block.border_style(Style::default().fg(Color::Yellow));
        }
        let npcs_list = List::new(npcs_items).block(npcs_block);
        frame.render_widget(npcs_list, chunks[1]);
        self.npcs_area = Some(chunks[1]);

        // 3. Group Members
        let group_items: Vec<ListItem> = state
            .game
            .group_members
            .iter()
            .map(|name| {
                ListItem::new(Span::styled(
                    format!("• {}", name),
                    Style::default().fg(Color::Green),
                ))
            })
            .collect();
        let group_list = List::new(group_items)
            .block(Block::default().borders(Borders::ALL).title(" Group Members "));
        frame.render_widget(group_list, chunks[2]);
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        _event_sender: &tokio::sync::mpsc::Sender<crate::events::ApplicationEvent>,
    ) -> bool {
        if state.ui.current_focus == crate::states::ui::GameFocus::NpcList {
            if let crossterm::event::Event::Key(key) = event {
                let npc_count = state.game.room_npcs.len();
                if npc_count == 0 {
                    return false;
                }

                match key.code {
                    crossterm::event::KeyCode::Up => {
                        let current = self.selected_npc_index.unwrap_or(0);
                        self.selected_npc_index = Some(if current == 0 { npc_count - 1 } else { current - 1 });
                        return true;
                    }
                    crossterm::event::KeyCode::Down => {
                        let current = self.selected_npc_index.unwrap_or(npc_count - 1);
                        self.selected_npc_index = Some(if current >= npc_count - 1 { 0 } else { current + 1 });
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
        false
    }
}
