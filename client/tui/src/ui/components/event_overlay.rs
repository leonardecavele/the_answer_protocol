use crate::states::app::AppState;
use crate::ui::components::Component;
use crossterm::event::{Event as CrosstermEvent, KeyCode, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear};

pub struct EventOverlayComponent {
    pub scroll: Option<u16>,
    pub last_max_scroll: u16,
}

impl EventOverlayComponent {
    pub fn new() -> Self {
        Self {
            scroll: None,
            last_max_scroll: 0,
        }
    }
}

impl Component for EventOverlayComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &tokio::sync::mpsc::Sender<crate::events::ApplicationEvent>,
    ) -> bool {
        if let CrosstermEvent::Key(key) = event {
            match key.code {
                KeyCode::Up => {
                    let current = self.scroll.unwrap_or(self.last_max_scroll);
                    self.scroll = Some(current.saturating_sub(1));
                    return true;
                }
                KeyCode::Down => {
                    let current = self.scroll.unwrap_or(self.last_max_scroll);
                    let new_scroll = current + 1;
                    if new_scroll >= self.last_max_scroll {
                        self.scroll = None;
                    } else {
                        self.scroll = Some(new_scroll);
                    }
                    return true;
                }
                KeyCode::Esc => {
                    state.ui.show_event_overlay = false;
                    return true;
                }
                _ => {}
            }
        } else if let CrosstermEvent::Mouse(mouse) = event {
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    let current = self.scroll.unwrap_or(self.last_max_scroll);
                    self.scroll = Some(current.saturating_sub(1));
                    return true;
                }
                MouseEventKind::ScrollDown => {
                    let current = self.scroll.unwrap_or(self.last_max_scroll);
                    let new_scroll = current + 1;
                    if new_scroll >= self.last_max_scroll {
                        self.scroll = None;
                    } else {
                        self.scroll = Some(new_scroll);
                    }
                    return true;
                }
                _ => {}
            }
        }
        
        true // Intercepte tous les événements sous l'overlay
    }

    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        if !state.ui.show_event_overlay {
            return;
        }

        let overlay_width = area.width * 80 / 100;
        let overlay_height = area.height * 80 / 100;
        let overlay_x = (area.width - overlay_width) / 2;
        let overlay_y = (area.height - overlay_height) / 2;

        let overlay_area = Rect {
            x: overlay_x,
            y: overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        let block = Block::default()
            .title(" Event History Overlay (Press Ctrl+E or Esc to close) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightMagenta));

        let inner_area = block.inner(overlay_area);
        let max_width = inner_area.width as usize;

        let lines = &state
            .ui
            .event_history
            .iter()
            .map(|line| format!("• {}\n", line))
            .collect::<Vec<_>>();

        let visual_lines = crate::ui::utils::wrap_slice_to_lines(lines, max_width);
        let lines_count = visual_lines.len() as u16;
        let inner_height = inner_area.height;
        
        let max_scroll = lines_count.saturating_sub(inner_height);
        self.last_max_scroll = max_scroll;
        
        let scroll_y = match self.scroll {
            None => max_scroll,
            Some(mut s) => {
                if s > max_scroll {
                    s = max_scroll;
                    self.scroll = Some(s);
                }
                s
            }
        };

        let paragraph = ratatui::widgets::Paragraph::new(visual_lines)
            .block(block)
            .scroll((scroll_y, 0));

        frame.render_widget(Clear, overlay_area);
        frame.render_widget(paragraph, overlay_area);
    }

    fn is_blocking(&self, state: &AppState) -> bool {
        state.ui.show_event_overlay
    }
}
