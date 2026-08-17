use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::OverlayKind;
use crate::ui::components::{Component, Lifecycle};
use crate::ui::theme::overlay_block;
use crate::ui::utils::{centered_rect, wrap_str_to_lines};
use api_client::commands::QuestData;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::widgets::Padding;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};
use tokio::sync::mpsc::Sender;

const POPUP_WIDTH_PERCENT: u16 = 60;
const POPUP_HEIGHT_PERCENT: u16 = 60;
const FOOTER_HEIGHT: u16 = 2;

pub struct QuestViewPopupComponent;

impl QuestViewPopupComponent {
    pub fn new() -> Self {
        Self
    }

    fn body(&self, quest: &QuestData, max_width: usize) -> Vec<Line<'static>> {
        let is_done = quest.status.eq_ignore_ascii_case("completed");
        let status_color = if is_done { Color::Green } else { Color::Yellow };

        let mut lines = vec![
            Line::from(Span::styled(
                format!("Status: {}", quest.status.to_lowercase()),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        lines.extend(wrap_str_to_lines(&quest.description, max_width));

        if !quest.reward.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Rewards",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));

            for reward in &quest.reward {
                lines.push(Line::from(format!(
                    "  {} x {} ({}%)",
                    reward.qty,
                    reward.r#type.to_lowercase(),
                    reward.chance
                )));
            }
        }

        lines
    }
}

impl Component for QuestViewPopupComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let quest_id = match state.game.ui.target_of(OverlayKind::QuestView) {
            Some(id) => id,
            None => return,
        };

        let quest = match state
            .game
            .player
            .quests
            .iter()
            .find(|q| q.quest_id == quest_id)
        {
            Some(quest) => quest,
            None => return,
        };

        let popup_area = centered_rect(
            area,
            (area.width * POPUP_WIDTH_PERCENT) / 100,
            (area.height * POPUP_HEIGHT_PERCENT) / 100,
        );

        frame.render_widget(Clear, popup_area);

        let block = overlay_block()
            .title(format!(" {} ", quest.quest_id))
            .padding(Padding::uniform(1))
            .style(Style::default().fg(Color::Yellow));

        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(FOOTER_HEIGHT)])
            .split(inner_area);

        let body = self.body(quest, chunks[0].width as usize);
        frame.render_widget(Paragraph::new(body), chunks[0]);

        let footer = Paragraph::new(" Press ESC or ENTER to close ")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(footer, chunks[1]);
    }
}

impl Lifecycle for QuestViewPopupComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if !state.game.ui.is_open(OverlayKind::QuestView) {
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
