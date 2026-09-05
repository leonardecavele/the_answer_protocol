use crate::events::ApplicationEvent;
use crate::renderer::components::{Component, EventFlow, Lifecycle};
use crate::renderer::layout::centered_rect_percent;
use crate::renderer::text::wrap_str_to_lines;
use crate::renderer::theme::{close_hint, popup_block, quest_status};
use crate::states::AppState;
use crate::states::game::QuestDetailState;
use api_client::commands::QuestData;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::widgets::Padding;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};
use tokio::sync::mpsc::Sender;

const POPUP_WIDTH_PERCENT: u16 = 60;
const POPUP_HEIGHT_PERCENT: u16 = 60;
const FOOTER_HEIGHT: u16 = 2;

#[derive(Default)]
pub struct QuestDetailPopup {
    area: Option<Rect>,
}

impl QuestDetailPopup {
    pub fn new() -> Self {
        Self::default()
    }

    fn body(&self, quest: &QuestData, max_width: usize) -> Vec<Line<'static>> {
        let (label, color) = quest_status(&quest.status);

        let mut lines = vec![
            Line::from(Span::styled(
                format!("Status: {}", label),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
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

impl Component for QuestDetailPopup {
    fn drawn_area(&self) -> Option<Rect> {
        self.area
    }

    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let quest_name = match state.game.overlays.get::<QuestDetailState>() {
            Some(overlay) => overlay.name.as_str(),
            None => return,
        };

        let quest = match state
            .game
            .player
            .quests
            .iter()
            .find(|q| q.name == quest_name)
        {
            Some(quest) => quest,
            None => return,
        };

        let popup_area = centered_rect_percent(area, POPUP_WIDTH_PERCENT, POPUP_HEIGHT_PERCENT);
        self.area = Some(popup_area);

        frame.render_widget(Clear, popup_area);

        let block = popup_block(format!(" {} ", quest.name)).padding(Padding::uniform(1));

        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(FOOTER_HEIGHT)])
            .split(inner_area);

        let body = self.body(quest, chunks[0].width as usize);
        frame.render_widget(Paragraph::new(body), chunks[0]);

        frame.render_widget(close_hint(), chunks[1]);
    }
}

impl Lifecycle for QuestDetailPopup {
    fn handle_device_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if !state.game.overlays.is_open::<QuestDetailState>() {
            return EventFlow::Ignored;
        }

        let CrosstermEvent::Key(key) = event else {
            return EventFlow::Ignored;
        };

        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                state.game.close_top_overlay();
                EventFlow::Consumed
            }
            _ => EventFlow::Ignored,
        }
    }
}
