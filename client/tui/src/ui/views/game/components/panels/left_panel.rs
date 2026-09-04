use crate::collections::Step;
use crate::data::manifest::NpcKind;
use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::GameFocus;
use crate::states::game::{
    ItemActionsState, ItemLocation, NpcActionsState, Overlay, QuestDetailState, Room,
};
use crate::ui::components::{Component, EventFlow, Lifecycle, is_mouse_in_rect};
use crate::ui::theme::{default_block, panel_block, quest_status, selection_style};
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

    fn draw_players(state: &AppState, room: &Room, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = room
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

        let list = List::new(items).block(default_block().title(" Room Players "));
        frame.render_widget(list, area);
    }

    fn draw_npcs(&mut self, state: &AppState, room: &Room, frame: &mut Frame, area: Rect) {
        let focused = state.game.focus() == GameFocus::NpcList;

        let items: Vec<ListItem> = room
            .npcs
            .iter()
            .enumerate()
            .map(|(index, npc)| {
                let color = match npc.kind {
                    NpcKind::Enemy => Color::Red,
                    NpcKind::QuestGiver => Color::Yellow,
                    NpcKind::Dialogue => Color::Blue,
                    NpcKind::Normal => Color::Reset,
                };
                let style = selection_style(color, focused && room.npcs.is_selected(index));

                ListItem::new(Span::styled(format!("• {} ({})", npc.name, npc.id), style))
            })
            .collect();

        let list = List::new(items).block(panel_block(" Room NPCs ", focused));
        frame.render_widget(list, area);
        self.npcs_area = Some(area);
    }

    fn draw_items(&mut self, state: &AppState, room: &Room, frame: &mut Frame, area: Rect) {
        let focused = state.game.focus() == GameFocus::RoomItemsList;

        let items: Vec<ListItem> = room
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let style = selection_style(Color::Cyan, focused && room.items.is_selected(index));

                ListItem::new(Span::styled(
                    format!("• {} ({})", item.name, item.id),
                    style,
                ))
            })
            .collect();

        let list = List::new(items).block(panel_block(" Room Items ", focused));
        frame.render_widget(list, area);
        self.items_area = Some(area);
    }

    fn draw_quests(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let focused = state.game.focus() == GameFocus::QuestList;
        let quests = &state.game.player.quests;

        let items: Vec<ListItem> = quests
            .iter()
            .enumerate()
            .map(|(index, quest)| {
                let (label, color) = quest_status(&quest.status);
                let style = selection_style(color, focused && quests.is_selected(index));

                ListItem::new(Span::styled(format!("[{}] {}", label, quest.name), style))
            })
            .collect();

        let list = List::new(items).block(panel_block(" Quests ", focused));
        frame.render_widget(list, area);
        self.quests_area = Some(area);
    }
}

impl Component for LeftPanel {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let Some(room) = &state.game.room else {
            return;
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
            ])
            .split(area);

        Self::draw_players(state, room, frame, chunks[0]);
        self.draw_npcs(state, room, frame, chunks[1]);
        self.draw_items(state, room, frame, chunks[2]);
        self.draw_quests(state, frame, chunks[3]);
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

        match state.game.focus() {
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
                    if !state.game.dialogue_cooldown_elapsed() {
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
