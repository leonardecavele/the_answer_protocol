use crate::collections::Step;
use crate::data::manifest::NpcKind;
use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::GameFocus;
use crate::states::game::Overlay::{ItemActions, NpcActions, QuestDetail};
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::components::lifecycle::EventFlow;
use crate::ui::theme::{default_block, panel_block};
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

impl Default for LeftPanel {
    fn default() -> Self {
        Self::new()
    }
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
            .map(|(idx, npc)| {
                let color = match npc.kind {
                    NpcKind::Enemy => Color::Red,
                    NpcKind::QuestGiver => Color::Yellow,
                    NpcKind::Dialogue => Color::Blue,
                    NpcKind::Normal => Color::Reset,
                };

                let mut style = Style::default().fg(color);

                if state.game.room.npcs.is_selected(idx) && state.game.focus == GameFocus::NpcList {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                ListItem::new(Span::styled(format!("• {} ({})", npc.name, npc.id), style))
            })
            .collect();
        let npcs_block = panel_block(" Room NPCs ", state.game.focus == GameFocus::NpcList);
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
            .map(|(idx, item)| {
                let mut style = Style::default().fg(Color::Cyan);

                if state.game.room.items.is_selected(idx)
                    && state.game.focus == GameFocus::RoomItemsList
                {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                ListItem::new(Span::styled(
                    format!("• {} ({})", item.name, item.id),
                    style,
                ))
            })
            .collect();
        let items_block = panel_block(" Room Items ", state.game.focus == GameFocus::RoomItemsList);
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
        let quests_block = panel_block(" Quests ", state.game.focus == GameFocus::QuestList);
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
    ) -> EventFlow {
        let key = match event {
            crossterm::event::Event::Key(key) => key,
            _ => return EventFlow::Ignored,
        };

        match state.game.focus {
            GameFocus::NpcList => match key.code {
                crossterm::event::KeyCode::Up => {
                    state.game.room.npcs.move_selection(Step::Previous);
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Down => {
                    state.game.room.npcs.move_selection(Step::Next);
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Enter => {
                    if !state.game.overlays.dialogue_cooldown_elapsed() {
                        return EventFlow::Consumed;
                    }

                    match state.game.room.npcs.selected() {
                        Some(npc) => {
                            state.game.overlays.open(NpcActions {
                                npc_id: npc.id.clone(),
                            });
                            EventFlow::Consumed
                        }
                        None => EventFlow::Ignored,
                    }
                }
                _ => EventFlow::Ignored,
            },
            GameFocus::RoomItemsList => match key.code {
                crossterm::event::KeyCode::Up => {
                    state.game.room.items.move_selection(Step::Previous);
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Down => {
                    state.game.room.items.move_selection(Step::Next);
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Enter => match state.game.room.items.selected() {
                    Some(item) => {
                        state.game.overlays.open(ItemActions {
                            item_id: item.id.clone(),
                        });
                        EventFlow::Consumed
                    }
                    None => EventFlow::Ignored,
                },
                _ => EventFlow::Ignored,
            },
            GameFocus::QuestList => match key.code {
                crossterm::event::KeyCode::Up => {
                    state.game.player.quests.move_selection(Step::Previous);
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Down => {
                    state.game.player.quests.move_selection(Step::Next);
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Enter => match state.game.player.quests.selected() {
                    Some(quest) => {
                        let quest_id = quest.quest_id.clone();
                        state.game.overlays.open(QuestDetail { quest_id });
                        EventFlow::Consumed
                    }
                    None => EventFlow::Ignored,
                },
                _ => EventFlow::Ignored,
            },
            _ => EventFlow::Ignored,
        }
    }
}
