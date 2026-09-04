use crate::events::ApplicationEvent;
use crate::events::types::NotificationType;
use crate::states::app::AppState;
use crate::ui::components::{Component, EventFlow, Lifecycle, is_mouse_in_rect};
use crate::ui::text::wrap_str_to_lines;
use crate::ui::theme::default_block;
use crossterm::event::{Event as CrosstermEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Clear, Gauge, Paragraph};
use tokio::sync::mpsc::Sender;

const MAX_VISIBLE_NOTIFICATIONS: usize = 5;
type NotificationID = String;

#[derive(Default)]
pub struct NotificationsOverlay {
    pub visible_areas: Vec<(NotificationID, Rect)>,
}

impl NotificationsOverlay {
    pub fn new() -> Self {
        Self {
            visible_areas: Vec::new(),
        }
    }
}

impl Component for NotificationsOverlay {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.visible_areas.clear();

        if state.ui.notifications.is_empty() {
            return;
        }

        let notifs_to_draw = state.ui.notifications.latest(MAX_VISIBLE_NOTIFICATIONS);

        let mut current_y = area.height;

        let mut max_width = (area.width as f32 * 0.4) as u16;
        max_width = max_width.max(40);
        if max_width > area.width {
            return;
        }

        let width = max_width;

        for notif in notifs_to_draw {
            let color = match notif.notification_type {
                NotificationType::Information => Color::Blue,
                NotificationType::Warning => Color::Yellow,
                NotificationType::Error => Color::Red,
                NotificationType::Success => Color::Green,
            };

            let block =
                default_block().style(Style::default().fg(color).add_modifier(Modifier::BOLD));

            let text = notif.message.as_str();
            let close = "[X]";

            let close_width = close.len() as u16;
            let inner_width = width.saturating_sub(2);

            let text_width = inner_width.saturating_sub(close_width + 1);

            let visual_lines = wrap_str_to_lines(text, text_width as usize);
            let lines_count = visual_lines.len() as u16;

            let mut height = lines_count + 2;
            if !notif.is_infinite() {
                height += 2;
            }

            let paragraph = Paragraph::new(visual_lines)
                .block(block)
                .alignment(Alignment::Left);

            let x = area.width.saturating_sub(width);

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

            let close_area = Rect {
                x: notif_area.x + notif_area.width - close_width - 1,
                y: notif_area.y + 1,
                width: close_width,
                height: 1,
            };

            frame.render_widget(
                Paragraph::new(close)
                    .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Right),
                close_area,
            );

            if let Some(remaining) = notif.remaining() {
                let remaining_fmt = format!("{:.1}s", remaining.as_secs_f32());
                let time_width = remaining_fmt.len() as u16 + 1;

                let gauge_area = Rect {
                    x: notif_area.x + 1,
                    y: notif_area.y + notif_area.height - 2,
                    width: notif_area.width.saturating_sub(2 + time_width),
                    height: 1,
                };

                let time_area = Rect {
                    x: gauge_area.x + gauge_area.width + 1,
                    y: gauge_area.y,
                    width: time_width,
                    height: 1,
                };

                let gauge = Gauge::default()
                    .gauge_style(Style::default().fg(color))
                    .label("")
                    .percent(notif.remaining_percent() as u16);

                frame.render_widget(gauge, gauge_area);

                frame.render_widget(
                    Paragraph::new(remaining_fmt).style(Style::default().fg(color)),
                    time_area,
                );
            };
        }
    }
}

impl Lifecycle for NotificationsOverlay {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if let CrosstermEvent::Mouse(MouseEvent {
            kind, column, row, ..
        }) = event
        {
            for (notification_id, area) in &self.visible_areas {
                let mouse_in_rect = is_mouse_in_rect(*column, *row, *area);

                if mouse_in_rect && *kind == MouseEventKind::Down(MouseButton::Left) {
                    state.ui.notifications.remove(notification_id);
                    return EventFlow::Consumed;
                }

                let Some(notification) = state.ui.notifications.get_mut(notification_id) else {
                    continue;
                };

                if mouse_in_rect {
                    notification.pause();
                } else {
                    notification.resume();
                }
            }
        }

        EventFlow::Ignored
    }
}
