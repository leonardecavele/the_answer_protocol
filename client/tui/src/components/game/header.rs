use crate::components::Component;
use crate::events::AppEvent;
use crate::state::AppState;
use crossterm::event::Event;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};

pub struct HeaderComponent;

#[async_trait::async_trait]
impl Component for HeaderComponent {
    async fn handle_event(
        &mut self,
        _state: &mut AppState,
        _event: &Event,
        _tx: &tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        // Header doesn't handle events
    }

    fn draw(&mut self, state: &mut AppState, f: &mut Frame, area: Rect) {
        let elapsed = if let Some(t) = state.net.connected_at {
            t.elapsed().as_secs()
        } else {
            0
        };
        let m = elapsed / 60;
        let s = elapsed % 60;

        let player_name = match &state.net.connection_state {
            crate::state::ConnectionState::Connected(n) => n.clone(),
            _ => "Unknown".to_string(),
        };

        let header_str = format!(
            " Time: {:02}:{:02} | Player: {} | Group: {} | HP: {}/{} | Online: {} ",
            m,
            s,
            player_name,
            state.game.group_name.as_deref().unwrap_or("None"),
            state.game.hp,
            state.game.max_hp,
            state.game.online_players
        );
        let header_widget = Paragraph::new(header_str)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .alignment(Alignment::Center);
        f.render_widget(header_widget, area);
    }
}
