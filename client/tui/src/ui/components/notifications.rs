use crate::states::app::AppState;
use crate::ui::components::Component;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use crate::events::types::NotificationType;

pub struct NotificationComponent;

impl NotificationComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for NotificationComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        if state.ui.notifications.is_empty() {
            return;
        }

        // Just display the most recent notification for simplicity in this implementation
        let notif = state.ui.notifications.last().unwrap();
        
        let color = match notif.notification_type {
            NotificationType::Information => Color::Blue,
            NotificationType::Warning => Color::Yellow,
            NotificationType::Error => Color::Red,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD));

        let paragraph = Paragraph::new(notif.message.clone())
            .block(block)
            .alignment(Alignment::Center);

        // Place it in the top right corner
        let width = std::cmp::max(notif.message.len() as u16 + 4, 30);
        let width = std::cmp::min(width, area.width);
        
        // Ensure we don't underflow
        let x = if area.width > width { area.width - width } else { 0 };
        
        let notif_area = Rect {
            x,
            y: 0,
            width,
            height: 3, // Enough for borders and 1 line of text
        };

        // Clear the area first so it draws on top of anything else
        frame.render_widget(Clear, notif_area);
        frame.render_widget(paragraph, notif_area);
    }
}
