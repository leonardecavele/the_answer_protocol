pub mod components;

use crate::events::{ApiEvent, ApplicationEvent};
use crate::states::app::AppState;
use crate::ui::components::scrollable::Scrollable;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;

use crate::states::game::{GameFocus, Overlay, OverlayKind};
use crate::ui::components::interactive::is_mouse_in_rect;
use crate::ui::views::game::components::{INVENTORY_ITEM_HEIGHT, INVENTORY_ITEM_WIDTH};
use api_client::events::{FightStartData, ServerEvent};
use components::{
    CenterPanelComponent, ChatOverlayComponent, DialoguePopupComponent, FooterComponent,
    HeaderComponent, HelpOverlayComponent, ItemPopupComponent, ItemViewPopupComponent,
    LeftPanelComponent, NpcActionPopup, QuestViewPopupComponent, RightPanelComponent,
};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use tokio::sync::mpsc;

pub struct GameView {
    header: HeaderComponent,
    footer: FooterComponent,
    left_panel: LeftPanelComponent,
    center_panel: CenterPanelComponent,
    right_panel: RightPanelComponent,
    chat_overlay: Scrollable<ChatOverlayComponent>,
    npc_popup: NpcActionPopup,
    item_popup: ItemPopupComponent,
    item_view_popup: ItemViewPopupComponent,
    quest_view_popup: QuestViewPopupComponent,
    dialogue_popup: Scrollable<DialoguePopupComponent>,
    help_overlay: Scrollable<HelpOverlayComponent>,
    right_panel_area: Option<Rect>,
    footer_area: Option<Rect>,
}

impl GameView {
    pub fn new() -> Self {
        Self {
            header: HeaderComponent::new(),
            footer: FooterComponent::new(),
            left_panel: LeftPanelComponent::new(),
            center_panel: CenterPanelComponent::new(),
            right_panel: RightPanelComponent::new(),
            chat_overlay: Scrollable::new(ChatOverlayComponent::new()),
            npc_popup: NpcActionPopup::new(),
            item_popup: ItemPopupComponent::new(),
            item_view_popup: ItemViewPopupComponent::new(),
            quest_view_popup: QuestViewPopupComponent::new(),
            dialogue_popup: Scrollable::new(DialoguePopupComponent::new()),
            help_overlay: Scrollable::new(HelpOverlayComponent::new()),
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
        let overlay_kinds: Vec<OverlayKind> = state.game.ui.overlays().map(Overlay::kind).collect();

        for kind in overlay_kinds {
            match kind {
                OverlayKind::Help => self.help_overlay.draw(state, frame, area),
                OverlayKind::Chat => self.chat_overlay.draw(state, frame, center_area),
                OverlayKind::NpcActions => self.npc_popup.draw(state, frame, area),
                OverlayKind::ItemActions => self.item_popup.draw(state, frame, area),
                OverlayKind::ItemView => self.item_view_popup.draw(state, frame, area),
                OverlayKind::QuestView => self.quest_view_popup.draw(state, frame, area),
                OverlayKind::Dialogue => self.dialogue_popup.draw(state, frame, area),
            }
        }
    }
}

impl Lifecycle for GameView {
    fn on_tick(&mut self, state: &mut AppState) {
        if state.game.ui.is_open(OverlayKind::Dialogue) {
            self.dialogue_popup.on_tick(state);
        }
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &mpsc::Sender<ApplicationEvent>,
    ) -> bool {
        if let Some(kind) = state.game.ui.top_kind() {
            let handled = match kind {
                OverlayKind::Help => {
                    self.help_overlay
                        .handle_terminal_event(state, event, event_sender)
                }
                OverlayKind::Chat => {
                    self.chat_overlay
                        .handle_terminal_event(state, event, event_sender)
                }
                OverlayKind::NpcActions => {
                    self.npc_popup
                        .handle_terminal_event(state, event, event_sender)
                }
                OverlayKind::ItemActions => {
                    self.item_popup
                        .handle_terminal_event(state, event, event_sender)
                }
                OverlayKind::ItemView => {
                    self.item_view_popup
                        .handle_terminal_event(state, event, event_sender)
                }
                OverlayKind::QuestView => {
                    self.quest_view_popup
                        .handle_terminal_event(state, event, event_sender)
                }
                OverlayKind::Dialogue => {
                    self.dialogue_popup
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
                state.game.ui.toggle(Overlay::Help);
                return true;
            }
        }

        if let CrosstermEvent::Key(key) = event {
            if key.code == KeyCode::F(1) {
                state.game.ui.toggle(Overlay::Chat);
                return true;
            }
            if key.code == KeyCode::Tab {
                state.game.ui.current_focus = match state.game.ui.current_focus {
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
                state.game.ui.current_focus = match state.game.ui.current_focus {
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
                if let Some(r) = self.left_panel.npcs_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.game.ui.current_focus = GameFocus::NpcList;

                        // Select the clicked NPC
                        let y = mouse.row.saturating_sub(r.y);
                        // y=0 is top border, y=1 is first item
                        if y > 0 {
                            let idx = (y - 1) as usize;
                            if idx < state.game.room.npcs.len() {
                                self.left_panel.selected_npc_index = Some(idx);
                            }
                        }
                    }
                }

                if let Some(r) = self.left_panel.items_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.game.ui.current_focus = GameFocus::RoomItemsList;

                        let y = mouse.row.saturating_sub(r.y);
                        if y > 0 {
                            let idx = (y - 1) as usize;
                            if idx < state.game.room.items.len() {
                                self.left_panel.selected_item_index = idx;
                            }
                        }
                    }
                }

                if let Some(r) = self.left_panel.quests_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.game.ui.current_focus = GameFocus::QuestList;

                        let y = mouse.row.saturating_sub(r.y);
                        if y > 0 {
                            let idx = (y - 1) as usize;
                            if idx < state.game.room.items.len() {
                                self.left_panel.selected_quest_index = idx;
                            }
                        }
                    }
                }

                if let Some(r) = self.center_panel.history_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.game.ui.current_focus = GameFocus::ActionHistory;
                    }
                }

                if let Some(r) = self.center_panel.inventory_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.game.ui.current_focus = GameFocus::InventoryGrid;

                        let rel_x = mouse.column.saturating_sub(r.x);
                        let rel_y = mouse.row.saturating_sub(r.y);
                        if rel_x > 0 && rel_y > 0 {
                            let col = (rel_x - 1) as usize / INVENTORY_ITEM_WIDTH as usize;
                            let row = (rel_y - 1) as usize / INVENTORY_ITEM_HEIGHT as usize;
                            let cols = self.center_panel.inventory.inventory_cols.max(1);
                            let idx = row * cols + col;
                            if idx < state.game.player.inventory.len() {
                                state.game.ui.inventory_cursor = idx;
                            }
                        }
                    }
                }

                if let Some(r) = self.right_panel_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.game.ui.current_focus = GameFocus::RightPanel;
                    }
                }
                if let Some(r) = self.footer_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.game.ui.current_focus = GameFocus::Input;
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
