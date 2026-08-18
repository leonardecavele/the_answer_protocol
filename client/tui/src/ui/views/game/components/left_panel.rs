use crate::collections::Step;
use crate::data::manifest::NpcType;
use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::GameFocus;
use crate::states::game::Overlay::{ItemActions, NpcActions, QuestDetail};
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

pub struct LeftPanel {
    pub npcs_area: Option<Rect>,
    pub items_area: Option<Rect>,
    pub quests_area: Option<Rect>,
}

impl LeftPanel {
    pub fn new() -> Self {
        Self {
            npcs_area: None,
            items_area: None,
            quests_area: None,
        }
    }
}

impl Component for LeftPanel {
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
            .enumerate()
            .map(|(idx, npc_id)| {
                let (display_name, npc_type) = match state.game.manifest.npcs.get(npc_id.as_str()) {
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

                if state.game.room.npcs.is_selected(idx)
                    && state.game.focus == GameFocus::NpcList
                {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                ListItem::new(Span::styled(
                    format!("• {} ({})", display_name, npc_id),
                    style,
                ))
            })
            .collect();
        let mut npcs_block = default_block().title(" Room NPCs ");
        if state.game.focus == GameFocus::NpcList {
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

                if state.game.room.items.is_selected(idx)
                    && state.game.focus == GameFocus::RoomItemsList
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
        if state.game.focus == GameFocus::RoomItemsList {
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

                if state.game.player.quests.is_selected(idx)
                    && state.game.focus == GameFocus::QuestList
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
        if state.game.focus == GameFocus::QuestList {
            quests_block = quests_block.border_style(Style::default().fg(Color::Yellow));
        }
        let quests_list = List::new(quests_items).block(quests_block);
        frame.render_widget(quests_list, chunks[3]);
        self.quests_area = Some(chunks[3]);
    }
}

impl Lifecycle for LeftPanel {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        let key = match event {
            crossterm::event::Event::Key(key) => key,
            _ => return false,
        };

        match state.game.focus {
            GameFocus::NpcList => match key.code {
                crossterm::event::KeyCode::Up => {
                    state.game.room.npcs.move_selection(Step::Previous);
                    true
                }
                crossterm::event::KeyCode::Down => {
                    state.game.room.npcs.move_selection(Step::Next);
                    true
                }
                crossterm::event::KeyCode::Enter => {
                    if !state.game.overlays.dialogue_cooldown_elapsed() {
                        return true;
                    }

                    match state.game.room.npcs.selected().cloned() {
                        Some(npc_id) => {
                            state.game.overlays.open(NpcActions { npc_id });
                            true
                        }
                        None => false,
                    }
                }
                _ => false,
            },
            GameFocus::RoomItemsList => match key.code {
                crossterm::event::KeyCode::Up => {
                    state.game.room.items.move_selection(Step::Previous);
                    true
                }
                crossterm::event::KeyCode::Down => {
                    state.game.room.items.move_selection(Step::Next);
                    true
                }
                crossterm::event::KeyCode::Enter => {
                    match state.game.room.items.selected().cloned() {
                        Some(item_id) => {
                            state.game.overlays.open(ItemActions { item_id });
                            true
                        }
                        None => false,
                    }
                }
                _ => false,
            },
            GameFocus::QuestList => match key.code {
                crossterm::event::KeyCode::Up => {
                    state.game.player.quests.move_selection(Step::Previous);
                    true
                }
                crossterm::event::KeyCode::Down => {
                    state.game.player.quests.move_selection(Step::Next);
                    true
                }
                crossterm::event::KeyCode::Enter => match state.game.player.quests.selected() {
                    Some(quest) => {
                        let quest_id = quest.quest_id.clone();
                        state.game.overlays.open(QuestDetail { quest_id });
                        true
                    }
                    None => false,
                },
                _ => false,
            },
            _ => false,
        }
    }
}
