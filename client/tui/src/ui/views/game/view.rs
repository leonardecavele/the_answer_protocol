use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::{Component, EventFlow, Lifecycle, Scrollable, ScrollableHit};
use crate::ui::layout::percent_of;

use crate::states::game::{
    ChatState, GameFocus, HelpState, ItemActionsState, ItemLocation, NpcActionsState, Overlay,
    OverlayKind, QuestDetailState,
};
use crate::ui::views::game::components::{
    ActionHistoryPanel, ChatOverlay, DialoguePopup, Footer, FooterHit, Header, HelpOverlay,
    InventoryPanel, InventoryPanelHit, ItemActionsPopup, ItemDetailPopup, LeftPanel, LeftPanelHit,
    NpcActionsPopup, QuestDetailPopup, RightPanel, RightPanelHit,
};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use tokio::sync::mpsc;

pub struct GameView {
    header: Header,
    footer: Footer,
    left_panel: LeftPanel,
    action_history: Scrollable<ActionHistoryPanel>,
    inventory: InventoryPanel,
    right_panel: RightPanel,
    chat: Scrollable<ChatOverlay>,
    npc_actions: NpcActionsPopup,
    item_actions: ItemActionsPopup,
    item_detail: ItemDetailPopup,
    quest_detail: QuestDetailPopup,
    dialogue: Scrollable<DialoguePopup>,
    help: Scrollable<HelpOverlay>,
}

impl Default for GameView {
    fn default() -> Self {
        Self::new()
    }
}

impl GameView {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            footer: Footer::new(),
            left_panel: LeftPanel::new(),
            action_history: Scrollable::new(ActionHistoryPanel::new()),
            inventory: InventoryPanel::new(),
            right_panel: RightPanel::new(),
            chat: Scrollable::new(ChatOverlay::new()),
            npc_actions: NpcActionsPopup::new(),
            item_actions: ItemActionsPopup::new(),
            item_detail: ItemDetailPopup::new(),
            quest_detail: QuestDetailPopup::new(),
            dialogue: Scrollable::new(DialoguePopup::new()),
            help: Scrollable::new(HelpOverlay::new()),
        }
    }

    fn dispatch_overlay(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        sender: &mpsc::Sender<ApplicationEvent>,
    ) -> EventFlow {
        if let Some(kind) = state.game.overlays.top_kind() {
            let flow = match kind {
                OverlayKind::Help => self.help.handle_terminal_event(state, event, sender),
                OverlayKind::Chat => self.chat.handle_terminal_event(state, event, sender),
                OverlayKind::NpcActions => {
                    self.npc_actions.handle_terminal_event(state, event, sender)
                }
                OverlayKind::ItemActions => self
                    .item_actions
                    .handle_terminal_event(state, event, sender),
                OverlayKind::ItemDetail => {
                    self.item_detail.handle_terminal_event(state, event, sender)
                }
                OverlayKind::QuestDetail => self
                    .quest_detail
                    .handle_terminal_event(state, event, sender),
                OverlayKind::Dialogue => self.dialogue.handle_terminal_event(state, event, sender),
            };

            if flow.is_consumed() || kind.is_modal() {
                return EventFlow::Consumed;
            }
        }

        EventFlow::Ignored
    }

    fn dispatch_children(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        sender: &mpsc::Sender<ApplicationEvent>,
    ) -> EventFlow {
        if self
            .footer
            .handle_terminal_event(state, event, sender)
            .is_consumed()
        {
            return EventFlow::Consumed;
        }

        if self
            .left_panel
            .handle_terminal_event(state, event, sender)
            .is_consumed()
        {
            return EventFlow::Consumed;
        }

        if self
            .action_history
            .handle_terminal_event(state, event, sender)
            .is_consumed()
        {
            return EventFlow::Consumed;
        }

        if self
            .inventory
            .handle_terminal_event(state, event, sender)
            .is_consumed()
        {
            return EventFlow::Consumed;
        }

        if self
            .right_panel
            .handle_terminal_event(state, event, sender)
            .is_consumed()
        {
            return EventFlow::Consumed;
        }

        EventFlow::Ignored
    }

    fn handle_overlay_keys(state: &mut AppState, event: &CrosstermEvent) -> EventFlow {
        if let CrosstermEvent::Key(key) = event {
            if key.code == KeyCode::Char('h')
                && key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
            {
                state.game.overlays.toggle(Overlay::Help(HelpState));
                return EventFlow::Consumed;
            }

            if key.code == KeyCode::F(1) {
                state.game.overlays.toggle(Overlay::Chat(ChatState));
                return EventFlow::Consumed;
            }
        }

        EventFlow::Ignored
    }

    fn handle_focus_keys(state: &mut AppState, event: &CrosstermEvent) -> EventFlow {
        if let CrosstermEvent::Key(key) = event {
            if key.code == KeyCode::Tab {
                state.game.focus_next();
                return EventFlow::Consumed;
            }
            if key.code == KeyCode::BackTab {
                state.game.focus_prev();
                return EventFlow::Consumed;
            }
        }

        EventFlow::Ignored
    }

    fn update_focus_from_mouse(&mut self, state: &mut AppState, event: &CrosstermEvent) {
        if let CrosstermEvent::Mouse(mouse) = event
            && mouse.kind
                == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
        {
            let left_hit = self.left_panel.hit(mouse.column, mouse.row);
            let inventory_hit = self.inventory.hit(mouse.column, mouse.row);

            match left_hit {
                LeftPanelHit::Npc(_) => state.game.set_focus(GameFocus::NpcList),
                LeftPanelHit::Item(_) => state.game.set_focus(GameFocus::RoomItemsList),
                LeftPanelHit::Quest(_) => state.game.set_focus(GameFocus::QuestList),
                LeftPanelHit::None => {}
            }

            if let ScrollableHit::Box = self.action_history.hit(mouse.column, mouse.row) {
                state.game.set_focus(GameFocus::ActionHistory);
            }

            if let InventoryPanelHit::Item(_) = inventory_hit {
                state.game.set_focus(GameFocus::InventoryGrid);
            }

            if let RightPanelHit::Image = self.right_panel.hit(mouse.column, mouse.row) {
                state.game.set_focus(GameFocus::RightPanel);
            }

            if let FooterHit::CommandInput = self.footer.hit(mouse.column, mouse.row) {
                state.game.set_focus(GameFocus::Input);
            }

            Self::request_overlay(state, left_hit, inventory_hit);
        }
    }

    fn request_overlay(
        state: &mut AppState,
        left_hit: LeftPanelHit,
        inventory_hit: InventoryPanelHit,
    ) {
        let dialogue_ready = state.game.dialogue_cooldown_elapsed();
        let mut requested = None;

        if let Some(room) = state.game.room.as_mut() {
            match left_hit {
                LeftPanelHit::Npc(index) => {
                    if room.npcs.is_selected(index) && dialogue_ready {
                        requested = room.npcs.selected().map(|npc| {
                            Overlay::NpcActions(NpcActionsState::new(npc.id.clone(), &npc.kind))
                        });
                    } else {
                        room.npcs.select_index(index);
                    }
                }
                LeftPanelHit::Item(index) => {
                    if room.items.is_selected(index) {
                        requested = room.items.selected().map(|item| {
                            Overlay::ItemActions(ItemActionsState::new(
                                item.id.clone(),
                                ItemLocation::Room,
                            ))
                        });
                    } else {
                        room.items.select_index(index);
                    }
                }
                LeftPanelHit::Quest(index) => {
                    if state.game.player.quests.is_selected(index) {
                        requested = state.game.player.quests.selected().map(|quest| {
                            Overlay::QuestDetail(QuestDetailState::new(quest.name.clone()))
                        });
                    } else {
                        state.game.player.quests.select_index(index);
                    }
                }
                LeftPanelHit::None => {}
            }
        }

        if let InventoryPanelHit::Item(Some(index)) = inventory_hit {
            if state.game.player.inventory.is_selected(index) {
                requested = state.game.player.inventory.selected().map(|item| {
                    Overlay::ItemActions(ItemActionsState::new(
                        item.id.clone(),
                        ItemLocation::Inventory,
                    ))
                });
            } else {
                state.game.player.inventory.select_index(index);
            }
        }

        if let Some(overlay) = requested
            && state.game.overlays.top_kind().is_none()
        {
            state.game.overlays.open(overlay);
        }
    }
}

impl Component for GameView {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);

        let available_height = vertical_chunks[1].height;
        let mut right_width_constraint = Constraint::Percentage(40);

        if let Some(desired_width) = self.right_panel.get_desired_width(state, available_height) {
            let max_width = percent_of(area.width, 40);
            let min_width = percent_of(area.width, 20);
            let final_width = desired_width.clamp(min_width, max_width);
            right_width_constraint = Constraint::Length(final_width);
        }

        let has_left_panel = state.network.is_connected && state.game.room.is_some();
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(if has_left_panel { 20 } else { 0 }),
                Constraint::Min(1),
                right_width_constraint,
            ])
            .split(vertical_chunks[1]);

        self.header.draw(state, frame, vertical_chunks[0]);
        self.left_panel.draw(state, frame, horizontal_chunks[0]);

        let center_vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(horizontal_chunks[1]);

        self.action_history
            .draw(state, frame, center_vertical_chunks[0]);
        self.inventory.draw(state, frame, center_vertical_chunks[1]);

        self.right_panel.draw(state, frame, horizontal_chunks[2]);
        self.footer.draw(state, frame, vertical_chunks[2]);

        let center_area = horizontal_chunks[1];
        let overlay_kinds: Vec<OverlayKind> =
            state.game.overlays.iter().map(Overlay::kind).collect();

        for kind in overlay_kinds {
            match kind {
                OverlayKind::Help => self.help.draw(state, frame, area),
                OverlayKind::Chat => self.chat.draw(state, frame, center_area),
                OverlayKind::NpcActions => self.npc_actions.draw(state, frame, area),
                OverlayKind::ItemActions => self.item_actions.draw(state, frame, area),
                OverlayKind::ItemDetail => self.item_detail.draw(state, frame, area),
                OverlayKind::QuestDetail => self.quest_detail.draw(state, frame, area),
                OverlayKind::Dialogue => self.dialogue.draw(state, frame, area),
            }
        }
    }
}

impl Lifecycle for GameView {
    fn on_tick(&mut self, state: &mut AppState, sender: &mpsc::Sender<ApplicationEvent>) {
        self.dialogue.on_tick(state, sender);
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        sender: &mpsc::Sender<ApplicationEvent>,
    ) -> EventFlow {
        if self.dispatch_overlay(state, event, sender).is_consumed() {
            return EventFlow::Consumed;
        }

        if Self::handle_overlay_keys(state, event).is_consumed() {
            return EventFlow::Consumed;
        }

        if Self::handle_focus_keys(state, event).is_consumed() {
            return EventFlow::Consumed;
        }

        self.update_focus_from_mouse(state, event);
        self.dispatch_children(state, event, sender)
    }
}
