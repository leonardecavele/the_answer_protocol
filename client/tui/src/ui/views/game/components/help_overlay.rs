use crate::states::app::AppState;
use crate::ui::components::Component;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub struct HelpOverlayComponent {
    scroll: u16,
}

impl HelpOverlayComponent {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }
}

impl Component for HelpOverlayComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        if !state.ui.show_help_overlay {
            self.scroll = 0;
            return;
        }

        // Create a centered box for the help menu
        let popup_width = 60;
        let popup_height = 20;

        let x = area.width.saturating_sub(popup_width) / 2;
        let y = area.height.saturating_sub(popup_height) / 2;

        let popup_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width: popup_width,
            height: popup_height,
        };

        // Clear the background
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Line::from(" Keyboard Shortcuts ").alignment(Alignment::Center))
            .title_bottom(Line::from(" Press Ctrl+H to close ").alignment(Alignment::Center));

        // Create lines of help text
        let lines = vec![
            Line::from(vec![Span::styled("Global", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from("  ctrl+c: quit game"),
            Line::from("  ctrl+h: toggle help"),
            Line::from("  f1: toggle chat overlay"),
            Line::from("  mouse click: focus panels (input, room npcs, image)"),
            Line::from(""),
            Line::from(vec![Span::styled("Input panel", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from("  enter: send command / focus right panel"),
            Line::from(""),
            Line::from(vec![Span::styled("Right panel (details/movement)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from("  up/down/left/right: move north/south/west/east"),
            Line::from("  enter: focus room npcs list"),
            Line::from(""),
            Line::from(vec![Span::styled("Room npcs list", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from("  up/down: select npc"),
            Line::from("  enter: open interaction menu"),
            Line::from(""),
            Line::from(vec![Span::styled("Interaction menus & chat", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from("  esc: close menus"),
            Line::from("  up/down: change action (talk/attack) / scroll chat"),
            Line::from("  enter: execute action / next dialogue"),
            Line::from(""),
            Line::from(vec![Span::styled("Text commands (input)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from("  connect : connect to server"),
            Line::from("  quit : disconnect and close the game"),
            Line::from("  look : look around the room"),
            Line::from("  move <NORTH|SOUTH|EAST|WEST> : move to direction"),
            Line::from("  who : see online players"),
            Line::from("  say <msg> : send a global message"),
            Line::from("  msg <name> <msg> : send a private message"),
            Line::from("  take <item> : take an item"),
            Line::from("  drop <item> : drop an item"),
            Line::from("  inv : view inventory"),
            Line::from("  status : view status"),
            Line::from("  talk <npc> : talk to npc"),
            Line::from("  attack <npc> : attack npc"),
            Line::from("  quest : view active quest details"),
            Line::from("  quests : list all active quests"),
            Line::from("  group_create : create a new group"),
            Line::from("  group_join <name> : join a player's group"),
            Line::from("  group_invite <name> : invite a player to your group"),
            Line::from("  group_leave : leave your current group"),
            Line::from(""),
            Line::from(vec![Span::styled("Hud & status", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from("  top right : your hp and max hp"),
            Line::from("  top right : online players count"),
            Line::from("  bottom right : notifications (disappear after 5s)"),
        ];

        let content_height = lines.len() as u16;
        let visible_height = popup_height.saturating_sub(2); // subtract borders

        // Clamp scroll to maximum possible
        let max_scroll = content_height.saturating_sub(visible_height);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }

        let content = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left)
            .scroll((self.scroll, 0));

        frame.render_widget(content, popup_area);
    }

    fn handle_terminal_event(
        &mut self,
        _state: &mut AppState,
        event: &crossterm::event::Event,
        _event_sender: &tokio::sync::mpsc::Sender<crate::events::ApplicationEvent>,
    ) -> bool {
        match event {
            crossterm::event::Event::Key(key) => {
                match key.code {
                    crossterm::event::KeyCode::Up => {
                        self.scroll = self.scroll.saturating_sub(1);
                        true
                    }
                    crossterm::event::KeyCode::Down => {
                        self.scroll = self.scroll.saturating_add(1);
                        true
                    }
                    crossterm::event::KeyCode::PageUp => {
                        self.scroll = self.scroll.saturating_sub(5);
                        true
                    }
                    crossterm::event::KeyCode::PageDown => {
                        self.scroll = self.scroll.saturating_add(5);
                        true
                    }
                    _ => false,
                }
            }
            crossterm::event::Event::Mouse(mouse) => {
                match mouse.kind {
                    crossterm::event::MouseEventKind::ScrollUp => {
                        self.scroll = self.scroll.saturating_sub(1);
                        true
                    }
                    crossterm::event::MouseEventKind::ScrollDown => {
                        self.scroll = self.scroll.saturating_add(1);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
