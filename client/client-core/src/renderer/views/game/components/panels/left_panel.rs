use crate::collections::Step;
use crate::data::manifest::NpcKind;
use crate::events::ApplicationEvent;
use crate::renderer::components::{
    CommandButton, Component, EventFlow, Lifecycle, is_mouse_in_rect,
};
use crate::renderer::theme::{panel_block, quest_status, selection_style};
use crate::states::app::AppState;
use crate::states::game::GameFocus;
use crate::states::game::{
    InvitationActionsState, ItemActionsState, ItemLocation, NpcActionsState, Overlay,
    PlayerActionsState, QuestDetailState, Room,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    text::Span,
    widgets::{List, ListItem},
};
use tokio::sync::mpsc::Sender;

pub enum LeftPanelHit {
    Player(usize),
    Invitation(usize),
    Npc(usize),
    Item(usize),
    Quest(usize),
    None,
}

pub struct LeftPanel {
    players_area: Option<Rect>,
    npcs_area: Option<Rect>,
    items_area: Option<Rect>,
    quests_area: Option<Rect>,
    invitations_area: Option<Rect>,
    quests_button: CommandButton,
}

impl Default for LeftPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl LeftPanel {
    pub fn new() -> Self {
        Self {
            players_area: None,
            npcs_area: None,
            items_area: None,
            quests_area: None,
            invitations_area: None,
            quests_button: CommandButton::new("QUESTS", "QUESTS"),
        }
    }

    fn hit_entry(area: Rect, column: u16, row: u16) -> Option<usize> {
        if !is_mouse_in_rect(column, row, area) || row <= area.y || row + 1 >= area.bottom() {
            return None;
        }

        Some((row - area.y - 1) as usize)
    }

    pub fn hit(&self, column: u16, row: u16) -> LeftPanelHit {
        if let Some(area) = self.players_area
            && let Some(index) = Self::hit_entry(area, column, row)
        {
            return LeftPanelHit::Player(index);
        }

        if let Some(area) = self.npcs_area
            && let Some(index) = Self::hit_entry(area, column, row)
        {
            return LeftPanelHit::Npc(index);
        }

        if let Some(area) = self.items_area
            && let Some(index) = Self::hit_entry(area, column, row)
        {
            return LeftPanelHit::Item(index);
        }

        if let Some(area) = self.quests_area
            && let Some(index) = Self::hit_entry(area, column, row)
        {
            return LeftPanelHit::Quest(index);
        }

        if let Some(area) = self.invitations_area
            && let Some(index) = Self::hit_entry(area, column, row)
        {
            return LeftPanelHit::Invitation(index);
        }

        LeftPanelHit::None
    }

    fn draw_players(&mut self, state: &AppState, room: &Room, frame: &mut Frame, area: Rect) {
        let focused = state.game.focus() == GameFocus::PlayerList;

        let items: Vec<ListItem> = room
            .players
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let color = if Some(name) == state.game.player.name.as_ref() {
                    Color::Yellow
                } else {
                    Color::Reset
                };

                let style = selection_style(color, focused && room.players.is_selected(index));

                ListItem::new(Span::styled(format!("• {}", name), style))
            })
            .collect();

        let list = List::new(items).block(panel_block(" Room Players ", focused));
        frame.render_widget(list, area);
        self.players_area = Some(area);
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

    fn draw_invitations(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let focused = state.game.focus() == GameFocus::InvitationList;
        let invitations = &state.game.group.invitations;

        let items: Vec<ListItem> = invitations
            .iter()
            .enumerate()
            .map(|(index, leader)| {
                let style =
                    selection_style(Color::Magenta, focused && invitations.is_selected(index));

                ListItem::new(Span::styled(format!("• {}", leader), style))
            })
            .collect();

        let list = List::new(items).block(panel_block(" Invited By ", focused));
        frame.render_widget(list, area);
        self.invitations_area = Some(area);
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

        let width = self.quests_button.width();

        if area.width > width + 2 {
            let button_area = Rect::new(area.right() - width - 1, area.y, width, 1);
            self.quests_button.draw(frame, button_area);
        } else {
            self.quests_button.hide();
        }
    }
}

impl Component for LeftPanel {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let Some(room) = &state.game.room else {
            return;
        };

        let invitations = state.game.group.invitations.len() as u16;
        let offset = if invitations > 0 { 1 } else { 0 };

        let mut constraints = Vec::with_capacity(5);

        if invitations > 0 {
            constraints.push(Constraint::Length(invitations + 2));
        }

        constraints.extend([Constraint::Fill(1); 4]);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        match chunks.first().filter(|_| invitations > 0) {
            Some(area) => self.draw_invitations(state, frame, *area),
            None => self.invitations_area = None,
        }

        self.draw_players(state, room, frame, chunks[offset]);
        self.draw_npcs(state, room, frame, chunks[offset + 1]);
        self.draw_items(state, room, frame, chunks[offset + 2]);
        self.draw_quests(state, frame, chunks[offset + 3]);
    }
}

impl Lifecycle for LeftPanel {
    fn handle_device_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if let crossterm::event::Event::Mouse(mouse) = event
            && mouse.kind
                == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
            && let Some(command) = self.quests_button.hit(mouse.column, mouse.row)
        {
            let _ = event_sender.try_send(ApplicationEvent::SendRawCommand(command.to_string()));
            return EventFlow::Consumed;
        }

        let key = match event {
            crossterm::event::Event::Key(key) => key,
            _ => return EventFlow::Ignored,
        };

        match state.game.focus() {
            GameFocus::PlayerList => match key.code {
                crossterm::event::KeyCode::Up => {
                    if let Some(room) = &mut state.game.room {
                        room.players.move_selection(Step::Previous);
                    }
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Down => {
                    if let Some(room) = &mut state.game.room {
                        room.players.move_selection(Step::Next);
                    }
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Enter => {
                    let selected = state
                        .game
                        .room
                        .as_ref()
                        .and_then(|room| room.players.selected())
                        .cloned();

                    match selected {
                        Some(player_name) => {
                            let can_invite = state
                                .game
                                .group
                                .is_leader(state.game.player.name.as_deref());

                            state.game.overlays.open(Overlay::PlayerActions(
                                PlayerActionsState::new(player_name, can_invite),
                            ));
                            EventFlow::Consumed
                        }
                        None => EventFlow::Ignored,
                    }
                }
                _ => EventFlow::Ignored,
            },
            GameFocus::InvitationList => match key.code {
                crossterm::event::KeyCode::Up => {
                    state.game.group.invitations.move_selection(Step::Previous);
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Down => {
                    state.game.group.invitations.move_selection(Step::Next);
                    EventFlow::Consumed
                }
                crossterm::event::KeyCode::Enter => {
                    match state.game.group.invitations.selected().cloned() {
                        Some(leader) => {
                            state.game.overlays.open(Overlay::InvitationActions(
                                InvitationActionsState::new(leader),
                            ));
                            EventFlow::Consumed
                        }
                        None => EventFlow::Ignored,
                    }
                }
                _ => EventFlow::Ignored,
            },
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
