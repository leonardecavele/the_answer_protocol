pub mod components;

use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::scrollable::Scrollable;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;

use crate::states::game::{GameFocus, Overlay, OverlayKind};
use crate::ui::components::interactive::is_mouse_in_rect;
use self::components::{
    CenterPanel, ChatOverlay, DialoguePopup, Footer, Header, HelpOverlay, INVENTORY_ITEM_HEIGHT,
    INVENTORY_ITEM_WIDTH, ItemActionsPopup, ItemDetailPopup, LeftPanel, NpcActionsPopup,
    QuestDetailPopup, RightPanel,
};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use tokio::sync::mpsc;

pub struct GameView {
    header: Header,
    footer: Footer,
    left_panel: LeftPanel,
    center_panel: CenterPanel,
    right_panel: RightPanel,
    chat: Scrollable<ChatOverlay>,
    npc_actions: NpcActionsPopup,
    item_actions: ItemActionsPopup,
    item_detail: ItemDetailPopup,
    quest_detail: QuestDetailPopup,
    dialogue: Scrollable<DialoguePopup>,
    help: Scrollable<HelpOverlay>,
    right_panel_area: Option<Rect>,
    footer_area: Option<Rect>,
}

impl GameView {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            footer: Footer::new(),
            left_panel: LeftPanel::new(),
            center_panel: CenterPanel::new(),
            right_panel: RightPanel::new(),
            chat: Scrollable::new(ChatOverlay::new()),
            npc_actions: NpcActionsPopup::new(),
            item_actions: ItemActionsPopup::new(),
            item_detail: ItemDetailPopup::new(),
            quest_detail: QuestDetailPopup::new(),
            dialogue: Scrollable::new(DialoguePopup::new()),
            help: Scrollable::new(HelpOverlay::new()),
            right_panel_area: None,
            footer_area: None,
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
            let max_width = (area.width * 40) / 100;
            let min_width = (area.width * 20) / 100;
            let final_width = desired_width.clamp(min_width, max_width);
            right_width_constraint = Constraint::Length(final_width);
        }

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Min(1),
                right_width_constraint,
            ])
            .split(vertical_chunks[1]);

        self.header.draw(state, frame, vertical_chunks[0]);
        self.left_panel.draw(state, frame, horizontal_chunks[0]);
        self.center_panel.draw(state, frame, horizontal_chunks[1]);
        self.right_panel.draw(state, frame, horizontal_chunks[2]);
        self.footer.draw(state, frame, vertical_chunks[2]);

        self.right_panel_area = Some(horizontal_chunks[2]);
        self.footer_area = Some(vertical_chunks[2]);

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
    fn on_tick(&mut self, state: &mut AppState) {
        if state.game.overlays.is_open(OverlayKind::Dialogue) {
            self.dialogue.on_tick(state);
        }
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &mpsc::Sender<ApplicationEvent>,
    ) -> bool {
        if let Some(kind) = state.game.overlays.top_kind() {
            let handled = match kind {
                OverlayKind::Help => self.help.handle_terminal_event(state, event, event_sender),
                OverlayKind::Chat => self.chat.handle_terminal_event(state, event, event_sender),
                OverlayKind::NpcActions => {
                    self.npc_actions
                        .handle_terminal_event(state, event, event_sender)
                }
                OverlayKind::ItemActions => {
                    self.item_actions
                        .handle_terminal_event(state, event, event_sender)
                }
                OverlayKind::ItemDetail => {
                    self.item_detail
                        .handle_terminal_event(state, event, event_sender)
                }
                OverlayKind::QuestDetail => {
                    self.quest_detail
                        .handle_terminal_event(state, event, event_sender)
                }
                OverlayKind::Dialogue => {
                    self.dialogue
                        .handle_terminal_event(state, event, event_sender)
                }
            };

            if handled || kind.is_modal() {
                return true;
            }
        }

        if let CrosstermEvent::Key(key) = event {
            if key.code == KeyCode::Char('h')
                && key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
            {
                state.game.overlays.toggle(Overlay::Help);
                return true;
            }
        }

        if let CrosstermEvent::Key(key) = event {
            if key.code == KeyCode::F(1) {
                state.game.overlays.toggle(Overlay::Chat);
                return true;
            }
            if key.code == KeyCode::Tab {
                state.game.focus = match state.game.focus {
                    GameFocus::Input => GameFocus::NpcList,
                    GameFocus::NpcList => GameFocus::RoomItemsList,
                    GameFocus::RoomItemsList => GameFocus::QuestList,
                    GameFocus::QuestList => GameFocus::ActionHistory,
                    GameFocus::ActionHistory => GameFocus::InventoryGrid,
                    GameFocus::InventoryGrid => GameFocus::RightPanel,
                    GameFocus::RightPanel => GameFocus::Input,
                };
                return true;
            }
            if key.code == KeyCode::BackTab {
                state.game.focus = match state.game.focus {
                    GameFocus::Input => GameFocus::RightPanel,
                    GameFocus::RightPanel => GameFocus::InventoryGrid,
                    GameFocus::InventoryGrid => GameFocus::ActionHistory,
                    GameFocus::ActionHistory => GameFocus::QuestList,
                    GameFocus::QuestList => GameFocus::RoomItemsList,
                    GameFocus::RoomItemsList => GameFocus::NpcList,
                    GameFocus::NpcList => GameFocus::Input,
                };
                return true;
            }
        }

        if let CrosstermEvent::Mouse(mouse) = event {
            if mouse.kind
                == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
            {
                if let Some(area) = self.left_panel.npcs_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, area) {
                        state.game.focus = GameFocus::NpcList;

                        let y = mouse.row.saturating_sub(area.y);
                        if y > 0 {
                            state.game.room.npcs.select_index((y - 1) as usize);
                        }
                    }
                }

                if let Some(area) = self.left_panel.items_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, area) {
                        state.game.focus = GameFocus::RoomItemsList;

                        let y = mouse.row.saturating_sub(area.y);
                        if y > 0 {
                            state.game.room.items.select_index((y - 1) as usize);
                        }
                    }
                }

                if let Some(area) = self.left_panel.quests_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, area) {
                        state.game.focus = GameFocus::QuestList;

                        let y = mouse.row.saturating_sub(area.y);
                        if y > 0 {
                            state.game.player.quests.select_index((y - 1) as usize);
                        }
                    }
                }

                if let Some(area) = self.center_panel.history_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, area) {
                        state.game.focus = GameFocus::ActionHistory;
                    }
                }

                if let Some(area) = self.center_panel.inventory_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, area) {
                        state.game.focus = GameFocus::InventoryGrid;

                        let rel_x = mouse.column.saturating_sub(area.x);
                        let rel_y = mouse.row.saturating_sub(area.y);
                        if rel_x > 0 && rel_y > 0 {
                            let col = (rel_x - 1) as usize / INVENTORY_ITEM_WIDTH as usize;
                            let row = (rel_y - 1) as usize / INVENTORY_ITEM_HEIGHT as usize;
                            let cols = self.center_panel.inventory.inventory_cols.max(1);
                            let idx = row * cols + col;
                            state.game.player.inventory.select_index(idx);
                        }
                    }
                }

                if let Some(area) = self.right_panel_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, area) {
                        state.game.focus = GameFocus::RightPanel;
                    }
                }
                if let Some(area) = self.footer_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, area) {
                        state.game.focus = GameFocus::Input;
                    }
                }
            }
        }

        if self
            .footer
            .handle_terminal_event(state, event, event_sender)
        {
            return true;
        }
        if self
            .header
            .handle_terminal_event(state, event, event_sender)
        {
            return true;
        }
        if self
            .left_panel
            .handle_terminal_event(state, event, event_sender)
        {
            return true;
        }
        if self
            .center_panel
            .handle_terminal_event(state, event, event_sender)
        {
            return true;
        }
        if self
            .right_panel
            .handle_terminal_event(state, event, event_sender)
        {
            return true;
        }
        false
    }
}
