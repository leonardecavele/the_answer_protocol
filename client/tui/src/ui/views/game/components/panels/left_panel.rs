use crate::collections::Step;
use crate::data::manifest::NpcKind;
use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::GameFocus;
use crate::states::game::{
    ItemActionsState, ItemLocation, NpcActionsState, Overlay, QuestDetailState,
};
use crate::ui::components::Lifecycle;
use crate::ui::components::component::Component;
use crate::ui::components::interactive::is_mouse_in_rect;
use crate::ui::components::lifecycle::EventFlow;
use crate::ui::theme::{default_block, panel_block, quest_status};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{List, ListItem},
};
use tokio::sync::mpsc::Sender;

pub enum LeftPanelHit {
    Npc(usize),
    Item(usize),
    Quest(usize),
    None,
}

#[derive(Default)]
pub struct LeftPanel {
    npcs_area: Option<Rect>,
    items_area: Option<Rect>,
    quests_area: Option<Rect>,
}

impl LeftPanel {
    pub fn new() -> Self {
        Self {
            npcs_area: None,
            items_area: None,
            quests_area: None,
        }
    }

    pub fn hit(&self, column: u16, row: u16) -> LeftPanelHit {
        if let Some(area) = self.npcs_area
            && is_mouse_in_rect(column, row, area)
        {
            return LeftPanelHit::Npc(row.saturating_sub(area.y).saturating_sub(1) as usize);
        }

        if let Some(area) = self.items_area
            && is_mouse_in_rect(column, row, area)
        {
            return LeftPanelHit::Item(row.saturating_sub(area.y).saturating_sub(1) as usize);
        }

        if let Some(area) = self.quests_area
            && is_mouse_in_rect(column, row, area)
        {
            return LeftPanelHit::Quest(row.saturating_sub(area.y).saturating_sub(1) as usize);
        }

        LeftPanelHit::None
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

        let Some(room) = &state.game.room else {
            return;
        };

        // 1. Room Players
        let players_items: Vec<ListItem> = room
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
        let npcs_items: Vec<ListItem> = room
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

                if room.npcs.is_selected(idx) && state.game.focus == GameFocus::NpcList {
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
        let items: Vec<ListItem> = room
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let mut style = Style::default().fg(Color::Cyan);

                if room.items.is_selected(idx) && state.game.focus == GameFocus::RoomItemsList {
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
                let (label, color) = quest_status(&q.status);

                let mut style = Style::default().fg(color);

                if state.game.player.quests.is_selected(idx)
                    && state.game.focus == GameFocus::QuestList
                {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                ListItem::new(Span::styled(format!("[{}] {}", label, q.name), style))
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
                    if let Some(room) = &mut state.game.room {
                        room.npcs.move_selection(Step::Previous);
                    }
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Down => {
                    if let Some(room) = &mut state.game.room {
                        room.npcs.move_selection(Step::Next);
                    }
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Enter => {
                    if !state.game.overlays.dialogue_cooldown_elapsed() {
                        return EventFlow::Consumed;
                    }

                    let selected = state
                        .game
                        .room
                        .as_ref()
                        .and_then(|room| room.npcs.selected())
                        .map(|npc| (npc.id.clone(), npc.kind.clone()));

                    match selected {
                        Some((npc_id, kind)) => {
                            state
                                .game
                                .overlays
                                .open(Overlay::NpcActions(NpcActionsState::new(npc_id, &kind)));
                            EventFlow::Consumed
                        }
                        None => EventFlow::Ignored,
                    }
                }
                _ => EventFlow::Ignored,
            },
            GameFocus::RoomItemsList => {
                match key.code {
                    crossterm::event::KeyCode::Up => {
                        if let Some(room) = &mut state.game.room {
                            room.items.move_selection(Step::Previous);
                        }
                        EventFlow::Consumed
                    }
                    crossterm::event::KeyCode::Down => {
                        if let Some(room) = &mut state.game.room {
                            room.items.move_selection(Step::Next);
                        }
                        EventFlow::Consumed
                    }
                    crossterm::event::KeyCode::Enter => {
                        let selected = state
                            .game
                            .room
                            .as_ref()
                            .and_then(|room| room.items.selected())
                            .map(|item| item.id.clone());

                        match selected {
                            Some(item_id) => {
                                state.game.overlays.open(Overlay::ItemActions(
                                    ItemActionsState::new(item_id, ItemLocation::Room),
                                ));
                                EventFlow::Consumed
                            }
                            None => EventFlow::Ignored,
                        }
                    }
                    _ => EventFlow::Ignored,
                }
            }
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
                        let name = quest.name.clone();
                        state
                            .game
                            .overlays
                            .open(Overlay::QuestDetail(QuestDetailState::new(name)));
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
