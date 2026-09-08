use crate::events::ApplicationEvent;
use crate::renderer::components::{Component, EventFlow, Lifecycle};
use crate::renderer::layout::{centered_rect, percent_of};
use crate::renderer::text::wrap_str_to_lines;
use crate::renderer::theme::{close_hint, dim_style, popup_block, quest_status};
use crate::states::AppState;
use crate::states::game::QuestDetailState;
use client_api::commands::{QuestData, QuestReward};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::widgets::{Borders, Padding};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
};
use tokio::sync::mpsc::Sender;

const POPUP_WIDTH_PERCENT: u16 = 60;
const MAX_HEIGHT_PERCENT: u16 = 60;
const MIN_HEIGHT: u16 = 8;
const BORDERS: u16 = 2;
const HORIZONTAL_PADDING: u16 = 2;
const STATUS_HEIGHT: u16 = 2;
const PROGRESS_HEIGHT: u16 = 1;
const SPACER_HEIGHT: u16 = 1;
const FOOTER_HEIGHT: u16 = 2;

#[derive(Default)]
pub struct QuestDetailPopup {
    area: Option<Rect>,
}

impl QuestDetailPopup {
    pub fn new() -> Self {
        Self::default()
    }

    fn reward_label(state: &AppState, quest: &QuestData, reward: &QuestReward) -> String {
        let item_name = state.game.manifest.item_name(&reward.r#type);
        let item = format!("{} x{}", item_name, reward.qty);

        if quest.is_completed() || reward.chance >= 100 {
            item
        } else {
            format!("{} ({}% chance)", item, reward.chance)
        }
    }

    fn draw_status(quest: &QuestData, frame: &mut Frame, area: Rect) {
        let (label, color) = quest_status(&quest.status);
        let steps = format!("{} / {}", quest.current_step, quest.max_step);
        let steps_width = steps.len() as u16;

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(steps_width)])
            .split(area);

        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {} ", label.to_uppercase()),
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::REVERSED | Modifier::BOLD),
            )),
            columns[0],
        );

        frame.render_widget(
            Paragraph::new(steps).alignment(Alignment::Right),
            columns[1],
        );
    }

    fn draw_progress(quest: &QuestData, frame: &mut Frame, area: Rect) {
        let (_, color) = quest_status(&quest.status);
        let percent = (u16::from(quest.current_step) * 100) / u16::from(quest.max_step);

        let block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(dim_style());

        let inner_area = block.inner(area);

        frame.render_widget(block, area);

        let filled_area = Rect {
            width: percent_of(inner_area.width, percent),
            ..inner_area
        };

        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black)),
            inner_area,
        );

        frame.render_widget(
            Block::default().style(Style::default().bg(color)),
            filled_area,
        );
    }

    fn body(state: &AppState, quest: &QuestData, max_width: usize) -> Vec<Line<'static>> {
        let mut lines = wrap_str_to_lines(&quest.description, max_width);

        if quest.reward.is_empty() {
            return lines;
        }

        let heading = if quest.is_completed() {
            "Rewards earned"
        } else {
            "Possible rewards"
        };

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            heading,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));

        for reward in &quest.reward {
            lines.push(Line::from(format!(
                "  {}",
                Self::reward_label(state, quest, reward)
            )));
        }

        lines
    }
}

impl Component for QuestDetailPopup {
    fn drawn_area(&self) -> Option<Rect> {
        self.area
    }

    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let quest_id = match state.game.overlays.get::<QuestDetailState>() {
            Some(overlay) => overlay.id,
            None => return,
        };

        let quest = match state.game.player.find_quest(quest_id) {
            Some(quest) => &quest.data,
            None => return,
        };

        let popup_width = percent_of(area.width, POPUP_WIDTH_PERCENT);
        let inner_width = popup_width.saturating_sub(BORDERS + HORIZONTAL_PADDING);

        let body = Self::body(state, quest, inner_width as usize);

        let content_height =
            STATUS_HEIGHT + PROGRESS_HEIGHT + SPACER_HEIGHT + body.len() as u16 + FOOTER_HEIGHT;

        let max_height = percent_of(area.height, MAX_HEIGHT_PERCENT).max(MIN_HEIGHT);
        let popup_height = (content_height + BORDERS).clamp(MIN_HEIGHT, max_height);

        let popup_area = centered_rect(area, popup_width, popup_height);
        self.area = Some(popup_area);

        frame.render_widget(Clear, popup_area);

        let block = popup_block(format!(" {} ", quest.name)).padding(Padding::horizontal(1));

        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(STATUS_HEIGHT),
                Constraint::Length(PROGRESS_HEIGHT),
                Constraint::Length(SPACER_HEIGHT),
                Constraint::Min(1),
                Constraint::Length(FOOTER_HEIGHT),
            ])
            .split(inner_area);

        Self::draw_status(quest, frame, chunks[0]);
        Self::draw_progress(quest, frame, chunks[1]);

        frame.render_widget(Paragraph::new(body), chunks[3]);
        frame.render_widget(close_hint(), chunks[4]);
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
