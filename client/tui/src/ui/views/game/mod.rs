pub mod components;

use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::Component;
use crate::ui::views::AppView;
use components::{
    CenterPanelComponent, ChatOverlayComponent, FooterComponent, HeaderComponent,
    LeftPanelComponent, RightPanelComponent,
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
    show_chat: bool,
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
            show_chat: false,
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
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &mpsc::Sender<ApplicationEvent>,
    ) {
        if let CrosstermEvent::Key(key) = event {
            if key.code == KeyCode::F(1) {
                self.show_chat = !self.show_chat;
                return;
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
