use crate::events::ApplicationEvent;
use crate::events::types::NotificationType;
use crate::states::app::AppState;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::components::interactive::is_mouse_in_rect;
use crate::ui::theme::default_block;
use crate::ui::utils::wrap_str_to_lines;
use crossterm::event::{Event as CrosstermEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Clear, Paragraph};
use std::cmp::min;
use tokio::sync::mpsc::Sender;

pub const MAX_VISIBLE_NOTIFICATIONS: usize = 5;
pub type NotificationID = String;

pub struct NotificationComponent {
    pub visible_areas: Vec<(NotificationID, Rect)>,
}

impl NotificationComponent {
    pub fn new() -> Self {
        Self {
            visible_areas: Vec::new(),
        }
    }
}

impl Component for NotificationComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.visible_areas.clear();

        if state.ui.notification.is_empty() {
            return;
        }

        // We take the last N notifications
        let notifs_to_draw = state.ui.notification.take(MAX_VISIBLE_NOTIFICATIONS);

        let mut current_y = area.height;

        for notif in notifs_to_draw {
            let color = match notif.notification_type {
                NotificationType::Information => Color::Blue,
                NotificationType::Warning => Color::Yellow,
                NotificationType::Error => Color::Red,
                NotificationType::Success => Color::Green,
            };

            let block =
                default_block().style(Style::default().fg(color).add_modifier(Modifier::BOLD));

            // Add "[X] " to indicate that it can be closed
            let text = format!("[X] {}", notif.message);

            let mut max_width = (area.width as f32 * 0.3) as u16;
            max_width = std::cmp::max(max_width, 40);

            let text_length = text.chars().count() as u16;

            let width = min(max_width, text_length + 2); // +2 for borders

            let inner_width = width.saturating_sub(2).max(1);

            let visual_lines = wrap_str_to_lines(&text, inner_width as usize);
            let lines_count = visual_lines.len() as u16;

            let height = lines_count + 2; // +2 for top/bottom borders

            let paragraph = Paragraph::new(visual_lines)
                .block(block)
                .alignment(Alignment::Left);

            let x = if area.width > width {
                area.width - width
            } else {
                0
            };

            if current_y < height {
                break;
            }

            current_y -= height;

            let notif_area = Rect {
                x,
                y: current_y,
                width,
                height,
            };

            self.visible_areas.push((notif.id.clone(), notif_area));

            frame.render_widget(Clear, notif_area);
            frame.render_widget(paragraph, notif_area);
        }
    }
}

impl Lifecycle for NotificationComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if let CrosstermEvent::Mouse(MouseEvent {
            kind, column, row, ..
        }) = event
        {
            if *kind == MouseEventKind::Down(MouseButton::Left) {
                // Find which notification was clicked
                if let Some((clicked_id, _)) = self
                    .visible_areas
                    .iter()
                    .find(|(_, area)| is_mouse_in_rect(*column, *row, *area))
                {
                    let id_to_remove = clicked_id.clone();
                    state.ui.notification.remove(&id_to_remove);
                    return true;
                }
            }
        }
        false
    }
}
