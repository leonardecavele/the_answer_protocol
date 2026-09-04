use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::{FightPhase, Sprite};
use crate::ui::components::{Component, EventFlow, Lifecycle};
use crate::ui::image::ImageRenderer;
use crate::ui::theme::{default_block, dim_style};
use api_client::ApiRequest;
use api_client::commands::FightAttackCommand;
use api_client::events::FightStartData;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use mpsc::Sender;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Gauge, Paragraph};
use ratatui_code_editor::editor::Editor;
use ratatui_code_editor::theme::vesper;
use ratatui_image::Resize;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
const EDITOR_LANGUAGE: &str = "c";
const FIGHT_END_GRACE: Duration = Duration::from_secs(10);
const EDITOR_BG: Color = Color::Rgb(0x16, 0x16, 0x16);
const HEADER_HEIGHT: u16 = 3;
const FOOTER_HEIGHT: u16 = 3;
const OPPONENT_WIDTH: u16 = 60;
const MIN_EDITOR_WIDTH: u16 = 80;
const NO_IMAGE: &str = " No image ";
const HEALTH_BAR_HEIGHT: u16 = 1;

pub struct EditorView {
    editor: Editor,
    npc_id: String,
    time: u64,
    nl_sep: String,
    sp_sep: String,
    started_at: Instant,
    editor_area: Rect,
    timed_out: bool,
    image_renderer: ImageRenderer,
}

impl EditorView {
    pub fn new(fight_data: &FightStartData) -> Result<Self, String> {
        let content = fight_data
            .code
            .replace(&fight_data.nl_sep, "\n")
            .replace(&fight_data.sp_sep, " ");

        let editor = Editor::new(EDITOR_LANGUAGE, &content, vesper())
            .map_err(|e| format!("Failed to open the code editor: {}", e))?;

        Ok(Self {
            editor,
            npc_id: fight_data.npc_id.clone(),
            time: fight_data.time,
            nl_sep: fight_data.nl_sep.clone(),
            sp_sep: fight_data.sp_sep.clone(),
            started_at: Instant::now(),
            editor_area: Rect::default(),
            timed_out: false,
            image_renderer: ImageRenderer::new(),
        })
    }

    fn serialize_code(&self) -> String {
        self.editor
            .get_content()
            .replace(" ", &self.sp_sep)
            .replace("\n", &self.nl_sep)
    }

    fn remaining_seconds(&self) -> u64 {
        self.time
            .saturating_sub(self.started_at.elapsed().as_secs())
    }

    fn grace_deadline(&self) -> Duration {
        Duration::from_secs(self.time) + FIGHT_END_GRACE
    }

    fn draw_opponent(&self, state: &AppState, frame: &mut Frame, area: Rect) {
        let block = default_block();
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(HEALTH_BAR_HEIGHT)])
            .split(inner);

        self.draw_sprite(state, frame, chunks[0]);
        Self::draw_health(state, frame, chunks[1]);
    }

    fn draw_sprite(&self, state: &AppState, frame: &mut Frame, area: Rect) {
        let sprite = Sprite::of_npc(&self.npc_id, &state.game.manifest);

        match sprite.frame_at(self.started_at.elapsed()) {
            Some(image_path) => {
                self.image_renderer
                    .draw_fitted(frame, area, image_path, Resize::Scale(None));
            }
            None => frame.render_widget(
                Paragraph::new(NO_IMAGE)
                    .alignment(Alignment::Center)
                    .style(dim_style()),
                area,
            ),
        }
    }

    fn draw_health(state: &AppState, frame: &mut Frame, area: Rect) {
        let Some(health) = state.game.fight.npc_health() else {
            return;
        };

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Red))
            .label(format!("{} / {}", health.current, health.max))
            .percent(health.percent());

        frame.render_widget(gauge, area);
    }

    fn header(&self, state: &AppState) -> Paragraph<'static> {
        let npc_name = state.game.manifest.npc_name(&self.npc_id);
        let remaining = self.remaining_seconds();

        let timer_color = if remaining == 0 {
            Color::Red
        } else {
            Color::Yellow
        };

        Paragraph::new(Span::styled(
            format!(
                "Fighting {}  -  {:02}:{:02}",
                npc_name,
                remaining / 60,
                remaining % 60
            ),
            Style::default()
                .fg(timer_color)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .block(default_block())
    }

    fn footer(&self, state: &AppState) -> Paragraph<'static> {
        let (text, style) = match state.game.fight.phase() {
            FightPhase::Editing => ("Press Ctrl+S to submit your code", dim_style()),
            FightPhase::AwaitingResult => (
                "Code submitted. Waiting for the other players...",
                dim_style(),
            ),
            FightPhase::Resolved { success: false } => (
                "Your code failed. Waiting for the other players...",
                Style::default().fg(Color::Red),
            ),
            FightPhase::Resolved { success: true } => (
                "Your code succeeded. Waiting for the other players...",
                Style::default().fg(Color::Green),
            ),
        };

        Paragraph::new(Span::styled(text, style))
            .alignment(Alignment::Center)
            .block(default_block())
    }
}

impl Component for EditorView {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(HEADER_HEIGHT),
                Constraint::Min(1),
                Constraint::Length(FOOTER_HEIGHT),
            ])
            .split(area);

        frame.render_widget(self.header(state), chunks[0]);
        frame.render_widget(self.footer(state), chunks[2]);

        let opponent_fits = chunks[1].width >= OPPONENT_WIDTH + MIN_EDITOR_WIDTH;
        let opponent_width = if opponent_fits { OPPONENT_WIDTH } else { 0 };

        let middle = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(opponent_width)])
            .split(chunks[1]);

        if opponent_fits {
            self.draw_opponent(state, frame, middle[1]);
        }

        self.editor_area = middle[0];
        frame.render_widget(
            Block::default().style(Style::default().bg(EDITOR_BG)),
            self.editor_area,
        );
        frame.render_widget(&self.editor, self.editor_area);

        if state.game.fight.phase() == FightPhase::Editing
            && let Some((x, y)) = self.editor.get_visible_cursor(&self.editor_area)
        {
            frame.set_cursor_position(Position::new(x, y));
        }
    }
}

impl Lifecycle for EditorView {
    fn on_tick(&mut self, _state: &mut AppState, sender: &Sender<ApplicationEvent>) {
        if self.timed_out || self.started_at.elapsed() < self.grace_deadline() {
            return;
        }

        self.timed_out = true;
        let _ = sender.try_send(ApplicationEvent::FightTimedOut);
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if state.game.fight.phase() != FightPhase::Editing {
            return EventFlow::Ignored;
        }

        let CrosstermEvent::Key(key) = event else {
            return EventFlow::Ignored;
        };

        if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
            let request = ApiRequest::FightAttack(FightAttackCommand {
                code: self.serialize_code(),
            });

            let _ = event_sender.try_send(ApplicationEvent::SendRequest(request));
            state.game.fight.submit();

            return EventFlow::Consumed;
        }

        let _ = self.editor.input(*key, &self.editor_area);

        EventFlow::Consumed
    }
}
