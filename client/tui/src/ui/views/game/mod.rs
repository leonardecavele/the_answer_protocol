pub mod components;

use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::components::scrollable::Scrollable;

use crate::states::ui::GameFocus;
use crate::ui::components::interactive::is_mouse_in_rect;
use components::{
    CenterPanelComponent, ChatOverlayComponent, DialoguePopupComponent, FooterComponent,
    HeaderComponent, HelpOverlayComponent, ItemPopupComponent, ItemViewPopupComponent, LeftPanelComponent,
    NpcActionPopup, RightPanelComponent,
};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
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
    dialogue_popup: Scrollable<DialoguePopupComponent>,
    help_overlay: Scrollable<HelpOverlayComponent>,
    show_chat: bool,
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
            dialogue_popup: Scrollable::new(DialoguePopupComponent::new()),
            help_overlay: Scrollable::new(HelpOverlayComponent::new()),
            show_chat: false,
            right_panel_area: None,
            footer_area: None,
        }
    }
}

impl Component for GameView {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        // Vertical layout: Header (3), Center (rest), Footer (3)
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

        // Horizontal layout for the center part
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

        if state.ui.show_help_overlay {
            self.help_overlay.draw(state, frame, area);
        }

        if self.show_chat {
            let center_area = horizontal_chunks[1];
            self.chat_overlay.draw(state, frame, center_area);
        }

        if state.ui.active_npc_popup.is_some() {
            self.npc_popup.draw(state, frame, area);
        }

        if state.ui.active_item_popup.is_some() {
            self.item_popup.draw(state, frame, area);
        }

        if state.game.active_dialogue.is_some() {
            self.dialogue_popup.draw(state, frame, area);
        }

        if state.ui.active_item_view_popup.is_some() {
            self.item_view_popup.draw(state, frame, area);
        }
    }
}

impl Lifecycle for GameView {
    fn on_tick(&mut self, state: &mut AppState) {
        if state.game.active_dialogue.is_some() {
            self.dialogue_popup.on_tick(state);
        }
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &mpsc::Sender<ApplicationEvent>,
    ) -> bool {
        if state.ui.show_help_overlay {
            self.help_overlay
                .handle_terminal_event(state, event, event_sender);
            return true;
        }

        if state.ui.active_item_view_popup.is_some() {
            if self
                .item_view_popup
                .handle_terminal_event(state, event, event_sender)
            {
                return true;
            }
        }

        if state.game.active_dialogue.is_some() {
            self.dialogue_popup
                .handle_terminal_event(state, event, event_sender);
            return true;
        }

        if state.ui.active_npc_popup.is_some() {
            self.npc_popup
                .handle_terminal_event(state, event, event_sender);
            return true;
        }

        if state.ui.active_item_popup.is_some() {
            self.item_popup
                .handle_terminal_event(state, event, event_sender);
            return true;
        }

        if let CrosstermEvent::Key(key) = event {
            if key.code == KeyCode::Char('h')
                && key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
            {
                state.ui.show_help_overlay = !state.ui.show_help_overlay;
                return true;
            }
        }

        if let CrosstermEvent::Key(key) = event {
            if key.code == KeyCode::F(1) {
                self.show_chat = !self.show_chat;
                return true;
            }
            if key.code == KeyCode::Tab {
                state.ui.current_focus = match state.ui.current_focus {
                    GameFocus::Input => GameFocus::ActionHistory,
                    GameFocus::ActionHistory => GameFocus::NpcList,
                    GameFocus::NpcList => GameFocus::RoomItemsList,
                    GameFocus::RoomItemsList => GameFocus::InventoryGrid,
                    GameFocus::InventoryGrid => GameFocus::RightPanel,
                    GameFocus::RightPanel => GameFocus::Input,
                };
                return true;
            }
            if key.code == KeyCode::BackTab {
                state.ui.current_focus = match state.ui.current_focus {
                    GameFocus::Input => GameFocus::RightPanel,
                    GameFocus::RightPanel => GameFocus::InventoryGrid,
                    GameFocus::InventoryGrid => GameFocus::RoomItemsList,
                    GameFocus::RoomItemsList => GameFocus::NpcList,
                    GameFocus::NpcList => GameFocus::ActionHistory,
                    GameFocus::ActionHistory => GameFocus::Input,
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
                        state.ui.current_focus = GameFocus::NpcList;

                        // Select the clicked NPC
                        let y = mouse.row.saturating_sub(r.y);
                        // y=0 is top border, y=1 is first item
                        if y > 0 {
                            let idx = (y - 1) as usize;
                            if idx < state.game.room_npcs.len() {
                                self.left_panel.selected_npc_index = Some(idx);
                            }
                        }
                    }
                }

                if let Some(r) = self.left_panel.items_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.ui.current_focus = GameFocus::RoomItemsList;

                        let y = mouse.row.saturating_sub(r.y);
                        if y > 0 {
                            let idx = (y - 1) as usize;
                            if idx < state.game.current_room_items.len() {
                                state.game.room_item_cursor = idx;
                            }
                        }
                    }
                }

                if let Some(r) = self.center_panel.history_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.ui.current_focus = GameFocus::ActionHistory;
                    }
                }

                if let Some(r) = self.center_panel.inventory_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.ui.current_focus = GameFocus::InventoryGrid;

                        let rel_x = mouse.column.saturating_sub(r.x);
                        let rel_y = mouse.row.saturating_sub(r.y);
                        if rel_x > 0 && rel_y > 0 {
                            let col = (rel_x - 1) as usize / 20; // item_width = 20
                            let row = (rel_y - 1) as usize / 10; // item_height = 10
                            let cols = self.center_panel.inventory.inventory_cols.max(1);
                            let idx = row * cols + col;
                            if idx < state.game.inventory.len() {
                                state.game.inventory_cursor = idx;
                            }
                        }
                    }
                }

                if let Some(r) = self.right_panel_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.ui.current_focus = GameFocus::RightPanel;
                    }
                }
                if let Some(r) = self.footer_area {
                    if is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.ui.current_focus = GameFocus::Input;
                    }
                }
            }
        }

        // Pass event to components (Chat overlay gets priority if visible)
        if self.show_chat {
            if self
                .chat_overlay
                .handle_terminal_event(state, event, event_sender)
            {
                return true;
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
