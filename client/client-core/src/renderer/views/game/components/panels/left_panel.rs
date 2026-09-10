use crate::collections::{SelectableList, Step};
use crate::events::{ApplicationEvent, SendEvent};
use crate::manifest::NpcKind;
use crate::renderer::components::{
    Component, EventFlow, LabelButton, Lifecycle, is_mouse_in_rect, scroll_direction,
};
use crate::renderer::theme::{
    ERROR_COLOR, INFORMATION_COLOR, INVITATION_COLOR, ITEM_COLOR, MUTED_COLOR, PLAYER_COLOR,
    WARNING_COLOR, panel_block, quest_status, selection_style,
};
use crate::states::AppState;
use crate::states::game::{
    GameFocus, InvitationActionsState, ItemActionsState, ItemLocation, NpcActionsState, Overlay,
    PlayerActionsState, QuestDetailState, Room,
};
use client_api::ApiRequest;
use client_api::commands::QuestsCommand;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    text::Span,
    widgets::{List, ListItem},
};
use tokio::sync::mpsc::Sender;

const BORDERS_HEIGHT: u16 = 2;

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
    quests_button: LabelButton,
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
            quests_button: LabelButton::new("QUESTS"),
        }
    }

    fn hit_entry(area: Option<Rect>, offset: usize, column: u16, row: u16) -> Option<usize> {
        let area = area?;

        if !is_mouse_in_rect(column, row, area) || row <= area.y || row + 1 >= area.bottom() {
            return None;
        }

        Some(offset + (row - area.y - 1) as usize)
    }

    fn visible_count(area: Option<Rect>) -> usize {
        area.map(|area| area.height.saturating_sub(BORDERS_HEIGHT) as usize)
            .unwrap_or(0)
    }

    pub fn hit(&self, state: &AppState, column: u16, row: u16) -> LeftPanelHit {
        if let Some(room) = &state.game.room {
            if let Some(index) =
                Self::hit_entry(self.players_area, room.players.offset(), column, row)
            {
                return LeftPanelHit::Player(index);
            }

            if let Some(index) = Self::hit_entry(self.npcs_area, room.npcs.offset(), column, row) {
                return LeftPanelHit::Npc(index);
            }

            if let Some(index) = Self::hit_entry(self.items_area, room.items.offset(), column, row)
            {
                return LeftPanelHit::Item(index);
            }
        }

        if let Some(index) = Self::hit_entry(
            self.quests_area,
            state.game.player.quests.offset(),
            column,
            row,
        ) {
            return LeftPanelHit::Quest(index);
        }

        if let Some(index) = Self::hit_entry(
            self.invitations_area,
            state.game.group.invitations.offset(),
            column,
            row,
        ) {
            return LeftPanelHit::Invitation(index);
        }

        LeftPanelHit::None
    }

    fn area_of(&self, focus: GameFocus) -> Option<Rect> {
        match focus {
            GameFocus::PlayerList => self.players_area,
            GameFocus::NpcList => self.npcs_area,
            GameFocus::RoomItemsList => self.items_area,
            GameFocus::QuestList => self.quests_area,
            GameFocus::InvitationList => self.invitations_area,
            _ => None,
        }
    }

    fn list_at(&self, column: u16, row: u16) -> Option<GameFocus> {
        [
            GameFocus::PlayerList,
            GameFocus::NpcList,
            GameFocus::RoomItemsList,
            GameFocus::QuestList,
            GameFocus::InvitationList,
        ]
        .into_iter()
        .find(|focus| {
            self.area_of(*focus)
                .is_some_and(|area| is_mouse_in_rect(column, row, area))
        })
    }

    fn set_visible_counts(&self, state: &mut AppState) {
        if let Some(room) = state.game.room.as_mut() {
            room.players
                .set_visible_count(Self::visible_count(self.area_of(GameFocus::PlayerList)));
            room.npcs
                .set_visible_count(Self::visible_count(self.area_of(GameFocus::NpcList)));
            room.items
                .set_visible_count(Self::visible_count(self.area_of(GameFocus::RoomItemsList)));
        }

        state
            .game
            .player
            .quests
            .set_visible_count(Self::visible_count(self.area_of(GameFocus::QuestList)));
        state
            .game
            .group
            .invitations
            .set_visible_count(Self::visible_count(self.area_of(GameFocus::InvitationList)));
    }

    fn scroll<T>(list: Option<&mut SelectableList<T>>, step: Step) {
        if let Some(list) = list {
            list.scroll(step, 1);
        }
    }

    fn scroll_list(&self, state: &mut AppState, focus: GameFocus, step: Step) {
        let room = state.game.room.as_mut();

        match focus {
            GameFocus::PlayerList => Self::scroll(room.map(|room| &mut room.players), step),
            GameFocus::NpcList => Self::scroll(room.map(|room| &mut room.npcs), step),
            GameFocus::RoomItemsList => Self::scroll(room.map(|room| &mut room.items), step),
            GameFocus::QuestList => Self::scroll(Some(&mut state.game.player.quests), step),
            GameFocus::InvitationList => {
                Self::scroll(Some(&mut state.game.group.invitations), step)
            }
            _ => {}
        }
    }

    fn draw_players(&mut self, state: &AppState, room: &Room, frame: &mut Frame, area: Rect) {
        let focused = state.game.focus() == GameFocus::PlayerList;

        let items: Vec<ListItem> = room
            .players
            .iter()
            .enumerate()
            .skip(room.players.offset())
            .map(|(index, name)| {
                let color = if Some(name) == state.game.player.name.as_ref() {
                    PLAYER_COLOR
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
            .skip(room.npcs.offset())
            .map(|(index, npc)| {
                let color = match npc.kind {
                    NpcKind::Enemy => ERROR_COLOR,
                    NpcKind::QuestGiver => WARNING_COLOR,
                    NpcKind::Dialogue => INFORMATION_COLOR,
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
            .skip(room.items.offset())
            .map(|(index, item)| {
                let style = selection_style(ITEM_COLOR, focused && room.items.is_selected(index));

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
            .skip(invitations.offset())
            .map(|(index, leader)| {
                let style =
                    selection_style(INVITATION_COLOR, focused && invitations.is_selected(index));

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
            .skip(quests.offset())
            .map(|(index, quest)| {
                let (_, color) = quest_status(&quest.data.status);
                let selected = focused && quests.is_selected(index);

                let progress = if quest.data.is_completed() {
                    "done".to_string()
                } else {
                    format!("{}/{}", quest.data.current_step, quest.data.max_step)
                };

                let color = if quest.data.is_completed() && !selected {
                    MUTED_COLOR
                } else {
                    color
                };

                let style = selection_style(color, selected);

                ListItem::new(Span::styled(
                    format!("• {} ({})", quest.data.name, progress),
                    style,
                ))
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
        self.set_visible_counts(state);

        if let crossterm::event::Event::Mouse(mouse) = event
            && mouse.kind
                == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
            && self.quests_button.hit(mouse.column, mouse.row)
        {
            let _ = event_sender.try_send(ApplicationEvent::Send(SendEvent::ApiRequest(
                ApiRequest::Quests(QuestsCommand),
            )));
            return EventFlow::Consumed;
        }

        if let crossterm::event::Event::Mouse(mouse) = event
            && let Some(step) = scroll_direction(mouse.kind)
            && let Some(focus) = self.list_at(mouse.column, mouse.row)
        {
            self.scroll_list(state, focus, step);
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
            GameFocus::RoomItemsList => match key.code {
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
                        .and_then(|room| room.items.selected());

                    match selected {
                        Some(item) => {
                            state
                                .game
                                .overlays
                                .open(Overlay::ItemActions(ItemActionsState::new(
                                    item.id.clone(),
                                    item.useable,
                                    ItemLocation::Room,
                                )));
                            EventFlow::Consumed
                        }
                        None => EventFlow::Ignored,
                    }
                }
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
                        let id = quest.id;
                        state
                            .game
                            .overlays
                            .open(Overlay::QuestDetail(QuestDetailState::new(id)));
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
