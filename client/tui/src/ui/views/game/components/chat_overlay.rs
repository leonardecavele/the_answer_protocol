use crate::states::app::AppState;
use crate::ui::components::Component;
use ratatui::{
    Frame,
    layout::Rect,
};

pub struct ChatOverlayComponent {
    scroll_offset: u16,
    last_max_scroll: u16,
}

impl ChatOverlayComponent {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            last_max_scroll: 0,
        }
    }
}

impl Component for ChatOverlayComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let block = crate::ui::theme::overlay_block().title(" Chat Overlay (F1 to hide) ");

        let inner_area = block.inner(area);
        let max_width = inner_area.width as usize;

        let mut visual_lines = Vec::new();

        for msg in &state.game.chat_history {
            let (prefix, color) = match &msg.channel {
                crate::states::game::ChatChannel::Global => {
                    ("[GLOBAL] ", ratatui::style::Color::Yellow)
                }
                crate::states::game::ChatChannel::Group => {
                    ("[GROUP] ", ratatui::style::Color::LightGreen)
                }
                crate::states::game::ChatChannel::Room => {
                    ("[ROOM] ", ratatui::style::Color::LightCyan)
                }
                crate::states::game::ChatChannel::Private(_) => {
                    ("[PRIVATE] ", ratatui::style::Color::LightMagenta)
                }
            };

            let full_text = format!("{}{}: {}", prefix, msg.sender, msg.content);
            let wrapped = textwrap::wrap(&full_text, max_width);

            for w in wrapped {
                visual_lines.push(
                    ratatui::text::Line::from(w.into_owned())
                        .style(ratatui::style::Style::default().fg(color)),
                );
            }
        }

        let lines_count = visual_lines.len() as u16;
        let inner_height = inner_area.height;

        let max_scroll = if lines_count > inner_height {
            lines_count - inner_height
        } else {
            0
        };

        self.last_max_scroll = max_scroll;

        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }

        let actual_scroll = max_scroll.saturating_sub(self.scroll_offset);

        let paragraph = ratatui::widgets::Paragraph::new(visual_lines)
            .block(block)
            .scroll((actual_scroll, 0));

        frame.render_widget(paragraph, area);
    }

    fn handle_terminal_event(
        &mut self,
        _state: &mut AppState,
        event: &crossterm::event::Event,
        _event_sender: &tokio::sync::mpsc::Sender<crate::events::ApplicationEvent>,
    ) -> bool {
        if let crossterm::event::Event::Key(key) = event {
            match key.code {
                crossterm::event::KeyCode::Up => {
                    self.scroll_offset = self
                        .scroll_offset
                        .saturating_add(1)
                        .min(self.last_max_scroll);
                    return true;
                }
                crossterm::event::KeyCode::Down => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                    return true;
                }
                crossterm::event::KeyCode::PageUp => {
                    self.scroll_offset = self
                        .scroll_offset
                        .saturating_add(10)
                        .min(self.last_max_scroll);
                    return true;
                }
                crossterm::event::KeyCode::PageDown => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(10);
                    return true;
                }
                _ => {}
            }
        }

        if let crossterm::event::Event::Mouse(mouse) = event {
            if mouse.kind == crossterm::event::MouseEventKind::ScrollUp {
                self.scroll_offset = self
                    .scroll_offset
                    .saturating_add(1)
                    .min(self.last_max_scroll);
                return true;
            } else if mouse.kind == crossterm::event::MouseEventKind::ScrollDown {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                return true;
            }
        }

        false
    }
}
