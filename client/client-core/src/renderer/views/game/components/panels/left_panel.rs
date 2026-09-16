use crate::collections::{SelectableList, Step};
use crate::events::{ApplicationEvent, SendEvent};
use crate::manifest::NpcKind;
use crate::renderer::components::{
    Component, EventFlow, LabelButton, Lifecycle, hit_row, is_mouse_in_rect,
};
use crate::renderer::text::truncate_to_width;
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
        hit_row(area, column, row).map(|row_index| offset + row_index)
    }

    fn open_focused_overlay(state: &mut AppState) -> EventFlow {
        match state.game.focus() {
            GameFocus::PlayerList => {
                let selected = state
                    .game
                    .room
                    .as_ref()
                    .and_then(|room| room.players.selected())
                    .cloned()
                    .filter(|player_name| !state.game.player.is_me(player_name));

                match selected {
                    Some(player_name) => {
                        let can_invite = state
                            .game
                            .group
                            .is_leader(state.game.player.name.as_deref());

                        state
                            .game
                            .overlays
                            .open(Overlay::PlayerActions(PlayerActionsState::new(
                                player_name,
                                can_invite,
                            )));
                        EventFlow::Consumed
                    }
                    None => EventFlow::Ignored,
                }
            }
            GameFocus::InvitationList => match state.game.group.invitations.selected().cloned() {
                Some(leader) => {
                    state.game.overlays.open(Overlay::InvitationActions(
                        InvitationActionsState::new(leader),
                    ));
                    EventFlow::Consumed
                }
                None => EventFlow::Ignored,
            },
            GameFocus::NpcList => {
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
            GameFocus::RoomItemsList => {
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
            GameFocus::QuestList => match state.game.player.quests.selected() {
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
        }
    }

    fn visible_count(area: Option<Rect>) -> usize {
        area.map(|area| area.height as usize).unwrap_or(0)
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

    fn move_selection<T>(list: Option<&mut SelectableList<T>>, step: Step) {
        if let Some(list) = list {
            list.move_selection(step);
        }
    }

    fn move_focused_selection(&self, state: &mut AppState, step: Step) -> EventFlow {
        let focus = state.game.focus();
        let room = state.game.room.as_mut();

        match focus {
            GameFocus::PlayerList => Self::move_selection(room.map(|room| &mut room.players), step),
            GameFocus::NpcList => Self::move_selection(room.map(|room| &mut room.npcs), step),
            GameFocus::RoomItemsList => {
                Self::move_selection(room.map(|room| &mut room.items), step)
            }
            GameFocus::QuestList => Self::move_selection(Some(&mut state.game.player.quests), step),
            GameFocus::InvitationList => {
                Self::move_selection(Some(&mut state.game.group.invitations), step)
            }
            _ => return EventFlow::Ignored,
        }

        EventFlow::Consumed
    }

    fn draw_players(&mut self, state: &AppState, room: &Room, frame: &mut Frame, area: Rect) {
        let focused = state.game.focus() == GameFocus::PlayerList;

        let items: Vec<ListItem> = room
            .players
            .iter()
            .enumerate()
            .skip(room.players.offset())
            .map(|(index, name)| {
                let is_me = state.game.player.is_me(name);

                let color = if is_me { PLAYER_COLOR } else { Color::Reset };
                let style = selection_style(color, focused && room.players.is_selected(index));

                let label = if is_me {
                    format!("• {} (You)", name)
                } else {
                    format!("• {}", name)
                };

                ListItem::new(Span::styled(
                    truncate_to_width(&label, area.width.saturating_sub(2) as usize),
                    style,
                ))
            })
            .collect();

        let title = format!(" Room Players ({}) ", room.players.len());
        let block = panel_block(title, focused);

        self.players_area = Some(block.inner(area));
        frame.render_widget(List::new(items).block(block), area);
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
                let label = format!("• {} ({})", npc.name, npc.id);

                ListItem::new(Span::styled(
                    truncate_to_width(&label, area.width.saturating_sub(2) as usize),
                    style,
                ))
            })
            .collect();

        let block = panel_block(" Room NPCs ", focused);

        self.npcs_area = Some(block.inner(area));
        frame.render_widget(List::new(items).block(block), area);
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
                let label = format!("• {} ({})", item.name, item.id);

                ListItem::new(Span::styled(
                    truncate_to_width(&label, area.width.saturating_sub(2) as usize),
                    style,
                ))
            })
            .collect();

        let block = panel_block(" Room Items ", focused);

        self.items_area = Some(block.inner(area));
        frame.render_widget(List::new(items).block(block), area);
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
                let label = format!("• {}", leader);

                ListItem::new(Span::styled(
                    truncate_to_width(&label, area.width.saturating_sub(2) as usize),
                    style,
                ))
            })
            .collect();

        let block = panel_block(" Invited By ", focused);

        self.invitations_area = Some(block.inner(area));
        frame.render_widget(List::new(items).block(block), area);
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
                let label = format!("• {} ({})", quest.data.name, progress);

                ListItem::new(Span::styled(
                    truncate_to_width(&label, area.width.saturating_sub(2) as usize),
                    style,
                ))
            })
            .collect();

        let block = panel_block(" Quests ", focused);

        self.quests_area = Some(block.inner(area));
        frame.render_widget(List::new(items).block(block), area);

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
    fn on_click(
        &mut self,
        state: &mut AppState,
        column: u16,
        row: u16,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        self.set_visible_counts(state);

        if self.quests_button.hit(column, row) {
            let _ = sender.try_send(ApplicationEvent::Send(SendEvent::ApiRequest(
                ApiRequest::Quests(QuestsCommand),
            )));

            return EventFlow::Consumed;
        }

        EventFlow::Ignored
    }

    fn on_scroll(&mut self, state: &mut AppState, step: Step, column: u16, row: u16) -> EventFlow {
        self.set_visible_counts(state);

        let Some(focus) = self.list_at(column, row) else {
            return EventFlow::Ignored;
        };

        self.scroll_list(state, focus, step);

        EventFlow::Consumed
    }

    fn on_key(
        &mut self,
        state: &mut AppState,
        key: &crossterm::event::KeyEvent,
        _sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        self.set_visible_counts(state);

        match key.code {
            crossterm::event::KeyCode::Up => self.move_focused_selection(state, Step::Previous),
            crossterm::event::KeyCode::Down => self.move_focused_selection(state, Step::Next),
            crossterm::event::KeyCode::Enter => Self::open_focused_overlay(state),
            _ => EventFlow::Ignored,
        }
    }
}
