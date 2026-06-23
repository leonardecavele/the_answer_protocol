use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::views::AppView;
use crossterm::event::Event as CrosstermEvent;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use tokio::sync::mpsc;

pub struct GameView;

impl GameView {
    pub fn new() -> Self {
        Self
    }
}

impl AppView for GameView {
    fn draw(&mut self, _state: &AppState, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Game View ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Green));

        let paragraph = Paragraph::new("Welcome to the Game!\nThe connection is established.")
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn handle_terminal_event(
        &mut self,
        _state: &mut AppState,
        _event: &CrosstermEvent,
        _event_sender: &mpsc::Sender<ApplicationEvent>,
    ) {
        // Handle game events later
    }
}
