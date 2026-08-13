use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::OverlayKind;
use crate::ui::components::{Component, Lifecycle};
use crate::ui::theme::overlay_block;
use crate::ui::utils::centered_rect;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::widgets::Padding;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Clear, Paragraph},
    Frame,
};
use std::vec;
use tokio::sync::mpsc::Sender;

const POPUP_WIDTH_PERCENT: u16 = 60;
const POPUP_HEIGHT_PERCENT: u16 = 60;
const FOOTER_HEIGHT: u16 = 2;

pub struct EditorViewPopupComponent;

impl EditorViewPopupComponent {
    pub fn new() -> Self {
        Self
    }

    fn body(&self, code: String, _max_width: usize) -> Vec<Line<'static>> {
        vec![Line::from(code)]
    }
}

impl Component for EditorViewPopupComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let code = match state.game.ui.target_of(OverlayKind::EditorView) {
            Some(code) => code,
            None => return,
        };

        let popup_area = centered_rect(
            area,
            (area.width * POPUP_WIDTH_PERCENT) / 100,
            (area.height * POPUP_HEIGHT_PERCENT) / 100,
        );

        frame.render_widget(Clear, popup_area);

        let block = overlay_block()
            .title("TODO")
            .padding(Padding::uniform(1))
            .style(Style::default().fg(Color::Yellow));

        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(FOOTER_HEIGHT)])
            .split(inner_area);

        let body = self.body(code.to_string(), chunks[0].width as usize);
        frame.render_widget(Paragraph::new(body), chunks[0]);

        let footer = Paragraph::new(" Press ESC or ENTER to close ")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(footer, chunks[1]);
    }
}

impl Lifecycle for EditorViewPopupComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if !state.game.ui.is_open(OverlayKind::EditorView) {
            return false;
        }

        if let CrosstermEvent::Key(key) = event {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                    state.game.ui.close_top();
                }
                _ => {}
            }
        }

        true
    }
}
