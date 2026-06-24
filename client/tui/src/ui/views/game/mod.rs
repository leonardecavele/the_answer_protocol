pub mod components;

use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::Component;
use crate::ui::views::AppView;
use components::{
    CenterPanelComponent, ChatOverlayComponent, FooterComponent, HeaderComponent,
    LeftPanelComponent, RightPanelComponent, NpcActionPopup, DialoguePopupComponent,
};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Clear;
use tokio::sync::mpsc;

pub struct GameView {
    header: HeaderComponent,
    footer: FooterComponent,
    left_panel: LeftPanelComponent,
    center_panel: CenterPanelComponent,
    right_panel: RightPanelComponent,
    chat_overlay: ChatOverlayComponent,
    npc_popup: NpcActionPopup,
    dialogue_popup: DialoguePopupComponent,
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
            chat_overlay: ChatOverlayComponent::new(),
            npc_popup: NpcActionPopup::new(),
            dialogue_popup: DialoguePopupComponent::new(),
            show_chat: false,
            right_panel_area: None,
            footer_area: None,
        }
    }
}

impl AppView for GameView {
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

        // Horizontal layout for the center part
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(40),
                Constraint::Percentage(40),
            ])
            .split(vertical_chunks[1]);

        self.header.draw(state, frame, vertical_chunks[0]);
        self.left_panel.draw(state, frame, horizontal_chunks[0]);
        self.center_panel.draw(state, frame, horizontal_chunks[1]);
        self.right_panel.draw(state, frame, horizontal_chunks[2]);
        self.footer.draw(state, frame, vertical_chunks[2]);

        self.right_panel_area = Some(horizontal_chunks[2]);
        self.footer_area = Some(vertical_chunks[2]);

        // Chat Overlay
        if self.show_chat {
            let center_area = horizontal_chunks[1];
            // Position chat overlay at the bottom right of the center area
            // Let's say it takes 50% width and 50% height of the center area
            let chat_width = center_area.width / 2;
            let chat_height = center_area.height / 2;
            let chat_area = Rect {
                x: center_area.x + center_area.width - chat_width,
                y: center_area.y + center_area.height - chat_height,
                width: chat_width,
                height: chat_height,
            };

            frame.render_widget(Clear, chat_area);
            self.chat_overlay.draw(state, frame, chat_area);
        }

        if state.ui.active_npc_popup.is_some() {
            self.npc_popup.draw(state, frame, area);
        }

        if state.game.active_dialogue.is_some() {
            self.dialogue_popup.draw(state, frame, area);
        }
    }

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
    ) {
        if state.game.active_dialogue.is_some() {
            self.dialogue_popup.handle_terminal_event(state, event, event_sender);
            return;
        }

        if state.ui.active_npc_popup.is_some() {
            self.npc_popup.handle_terminal_event(state, event, event_sender);
            return;
        }

        if let CrosstermEvent::Key(key) = event {
            if key.code == KeyCode::F(1) {
                self.show_chat = !self.show_chat;
                return;
            }
            if key.code == KeyCode::Tab {
                state.ui.current_focus = match state.ui.current_focus {
                    crate::states::ui::GameFocus::Input => crate::states::ui::GameFocus::NpcList,
                    crate::states::ui::GameFocus::NpcList => crate::states::ui::GameFocus::RightPanel,
                    crate::states::ui::GameFocus::RightPanel => crate::states::ui::GameFocus::Input,
                };
                return;
            }
            if key.code == KeyCode::BackTab {
                state.ui.current_focus = match state.ui.current_focus {
                    crate::states::ui::GameFocus::Input => crate::states::ui::GameFocus::RightPanel,
                    crate::states::ui::GameFocus::RightPanel => crate::states::ui::GameFocus::NpcList,
                    crate::states::ui::GameFocus::NpcList => crate::states::ui::GameFocus::Input,
                };
                return;
            }
        }

        if let CrosstermEvent::Mouse(mouse) = event {
            if mouse.kind == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                if let Some(r) = self.left_panel.npcs_area {
                    if crate::ui::components::is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.ui.current_focus = crate::states::ui::GameFocus::NpcList;
                        
                        // Select the clicked NPC
                        let rel_y = mouse.row.saturating_sub(r.y);
                        // The border is at rel_y == 0
                        if rel_y > 0 {
                            let index = (rel_y - 1) as usize;
                            if index < state.game.room_npcs.len() {
                                self.left_panel.selected_npc_index = Some(index);
                            }
                        }
                    }
                }
                if let Some(r) = self.right_panel_area {
                    if crate::ui::components::is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.ui.current_focus = crate::states::ui::GameFocus::RightPanel;
                    }
                }
                if let Some(r) = self.footer_area {
                    if crate::ui::components::is_mouse_in_rect(mouse.column, mouse.row, r) {
                        state.ui.current_focus = crate::states::ui::GameFocus::Input;
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
                return;
            }
        }

        if self
            .footer
            .handle_terminal_event(state, event, event_sender)
        {
            return;
        }
        if self
            .header
            .handle_terminal_event(state, event, event_sender)
        {
            return;
        }
        if self
            .left_panel
            .handle_terminal_event(state, event, event_sender)
        {
            return;
        }
        if self
            .center_panel
            .handle_terminal_event(state, event, event_sender)
        {
            return;
        }
        if self
            .right_panel
            .handle_terminal_event(state, event, event_sender)
        {
            return;
        }
    }
}
