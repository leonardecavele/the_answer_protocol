use crate::commands::handle_command;
use crate::components::Component;
use crate::events::AppEvent;
use crate::state::{AppState, GameFocus};
use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

pub struct InputComponent;

#[async_trait::async_trait]
impl Component for InputComponent {
    async fn handle_event(
        &mut self,
        state: &mut AppState,
        event: &Event,
        tx: &tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => {
                    state.ui.game_scroll_offset = state.ui.game_scroll_offset.saturating_add(1);
                }
                KeyCode::Down => {
                    state.ui.game_scroll_offset = state.ui.game_scroll_offset.saturating_sub(1);
                }
                KeyCode::Enter => {
                    let cmd_str = state.ui.input.value().to_string();
                    state.ui.input.reset();
                    if !cmd_str.trim().is_empty() {
                        handle_command(state, cmd_str, tx.clone());
                    }
                }
                _ => {
                    tui_input::backend::crossterm::EventHandler::handle_event(
                        &mut state.ui.input,
                        event,
                    );
                }
            }
        }
    }

    fn draw(&mut self, state: &mut AppState, f: &mut Frame, area: Rect) {
        let is_focused = matches!(state.ui.game_focus, GameFocus::Input);
        let unfocused_color = Color::DarkGray;
        let focused_color = Color::Yellow;

        let input_style = if is_focused {
            Style::default()
                .fg(focused_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(unfocused_color)
        };
        let input_widget = Paragraph::new(state.ui.input.value())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(input_style)
                    .title(" Command Input [Tab to switch] "),
            )
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(input_widget, area);
    }
}
