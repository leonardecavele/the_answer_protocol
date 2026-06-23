use crate::components::Component;
use crate::events::AppEvent;
use crate::state::AppState;
use crossterm::event::Event;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub struct EventsComponent;

#[async_trait::async_trait]
impl Component for EventsComponent {
    async fn handle_event(
        &mut self,
        _state: &mut AppState,
        _event: &Event,
        _tx: &tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        // Events overlay doesn't handle keyboard directly, InputComponent scrolls it via AppState
    }

    fn draw(&mut self, state: &mut AppState, f: &mut Frame, area: Rect) {
        let game_lines: Vec<Line> = state
            .game
            .game_output
            .iter()
            .map(|l| Line::from(l.as_str()))
            .collect();
        
        let max_scroll = (game_lines.len() as u16).saturating_sub(area.height.saturating_sub(2));
        let scroll = max_scroll.saturating_sub(state.ui.game_scroll_offset);

        let unfocused_color = Color::DarkGray;

        let messages_widget = Paragraph::new(game_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(unfocused_color))
                    .title(" Game Events "),
            )
            .scroll((scroll, 0))
            .wrap(Wrap { trim: true });
        f.render_widget(messages_widget, area);
    }
}
