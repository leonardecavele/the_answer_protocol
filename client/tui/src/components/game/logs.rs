use crate::components::Component;
use crate::events::AppEvent;
use crate::state::{AppState, GameFocus};
use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear},
};

pub struct LogsComponent;

#[async_trait::async_trait]
impl Component for LogsComponent {
    async fn handle_event(
        &mut self,
        state: &mut AppState,
        event: &Event,
        _tx: &tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => {
                    state
                        .ui
                        .logger_state
                        .transition(tui_logger::TuiWidgetEvent::UpKey);
                }
                KeyCode::Down => {
                    state
                        .ui
                        .logger_state
                        .transition(tui_logger::TuiWidgetEvent::DownKey);
                }
                KeyCode::Left => {
                    state
                        .ui
                        .logger_state
                        .transition(tui_logger::TuiWidgetEvent::LeftKey);
                }
                KeyCode::Right => {
                    state
                        .ui
                        .logger_state
                        .transition(tui_logger::TuiWidgetEvent::RightKey);
                }
                _ => {}
            }
        }
    }

    fn draw(&mut self, state: &mut AppState, f: &mut Frame, area: Rect) {
        if matches!(state.ui.game_focus, GameFocus::SystemLogs) {
            let overlay_width = area.width.saturating_mul(80) / 100;
            let overlay_height = area.height.saturating_mul(80) / 100;
            let overlay_x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
            let overlay_y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
            let overlay_rect = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

            let logs_widget = tui_logger::TuiLoggerWidget::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )
                        .title(" System Logs [Tab to hide] "),
                )
                .state(&state.ui.logger_state)
                .style_error(Style::default().fg(Color::Red))
                .style_info(Style::default().fg(Color::Blue));

            f.render_widget(Clear, overlay_rect);
            f.render_widget(logs_widget, overlay_rect);
        }
    }
}
