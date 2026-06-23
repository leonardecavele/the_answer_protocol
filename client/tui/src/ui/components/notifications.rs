use crate::constants::MAX_VISIBLE_NOTIFICATIONS;
use crate::events::types::NotificationType;
use crate::states::app::AppState;
use crate::ui::components::Component;
use crossterm::event::{Event as CrosstermEvent, MouseEvent, MouseEventKind};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

pub struct NotificationComponent {
    /// Stores the area (Rect) associated with the ID (String) of each visible notification
    pub visible_areas: Vec<(String, Rect)>,
}

impl NotificationComponent {
    pub fn new() -> Self {
        Self {
            visible_areas: Vec::new(),
        }
    }
}

impl Component for NotificationComponent {
    fn is_clickable(&self) -> bool {
        true
    }

    /// Override is_mouse_over to check the visible_areas array
    /// instead of relying on a single `last_area`.
    fn is_mouse_over(&self, col: u16, row: u16) -> bool {
        self.visible_areas.iter().any(|(_, area)| {
            col >= area.x && col < area.x + area.width && row >= area.y && row < area.y + area.height
        })
    }

    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.visible_areas.clear();

        if state.ui.notifications.is_empty() {
            return;
        }

        // We take the last N notifications
        let notifs_to_draw = state
            .ui
            .notifications
            .iter()
            .rev()
            .take(MAX_VISIBLE_NOTIFICATIONS)
            .collect::<Vec<_>>();

        let mut current_y = 0;

        for notif in notifs_to_draw {
            let color = match notif.notification_type {
                NotificationType::Information => Color::Blue,
                NotificationType::Warning => Color::Yellow,
                NotificationType::Error => Color::Red,
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(color).add_modifier(Modifier::BOLD));

            // Add "[X] " to indicate that it can be closed
            let text = format!("[X] {}", notif.message);
            
            // Dynamic width (max 30% of terminal width, minimum 20 chars)
            let mut max_width = (area.width as f32 * 0.3) as u16;
            max_width = std::cmp::max(max_width, 20);
            
            let text_length = text.chars().count() as u16;
            
            // Limit width if text is smaller than max_width
            let width = std::cmp::min(max_width, text_length + 2); // +2 for borders
            
            // Dynamic height calculation
            let inner_width = width.saturating_sub(2).max(1);
            
            // Integer math ceiling formula: (A + B - 1) / B
            let mut lines = (text_length + inner_width - 1) / inner_width;
            
            // Account for manual newlines
            lines += text.matches('\n').count() as u16;
            
            let height = lines + 2; // +2 for top/bottom borders

            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });

            // Position at top right
            let x = if area.width > width { area.width - width } else { 0 };

            let notif_area = Rect {
                x,
                y: current_y,
                width,
                height,
            };

            // Memorize the clickable area
            self.visible_areas.push((notif.id.clone(), notif_area));

            frame.render_widget(Clear, notif_area);
            frame.render_widget(paragraph, notif_area);

            // Shift the next notification downwards
            current_y += height; 
            
            // Protection against overflow
            if current_y > area.height {
                break; 
            }
        }
    }

    fn handle_terminal_event(&mut self, state: &mut AppState, event: &CrosstermEvent, _event_sender: &tokio::sync::mpsc::Sender<crate::events::ApplicationEvent>) -> bool {
        if let CrosstermEvent::Mouse(MouseEvent { kind, column, row, .. }) = event {
            if *kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                // Find which notification was clicked
                if let Some((clicked_id, _)) = self.visible_areas.iter().find(|(_, area)| {
                    *column >= area.x
                        && *column < area.x + area.width
                        && *row >= area.y
                        && *row < area.y + area.height
                }) {
                    // Remove the targeted notification from the state
                    let id_to_remove = clicked_id.clone();
                    state.ui.notifications.retain(|n| n.id != id_to_remove);
                    return true; // Event consumed
                }
            }
        }
        false
    }
}
