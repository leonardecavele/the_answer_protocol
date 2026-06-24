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

pub struct LeftPanelComponent;

impl LeftPanelComponent {
    pub fn new() -> Self {
        Self
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
                    NpcType::Normal => Color::Gray,
                };

                ListItem::new(Span::styled(
                    format!("• {}", display_name),
                    Style::default().fg(color),
                ))
            })
            .collect();
        let npcs_list = List::new(npcs_items)
            .block(Block::default().borders(Borders::ALL).title(" Room NPCs "));
        frame.render_widget(npcs_list, chunks[1]);

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
}
