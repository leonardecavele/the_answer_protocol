use crate::events::{ApplicationEvent, CustomEvent, SendEvent};
use crate::renderer::components::{Button, Component, EventFlow, Lifecycle};
use crate::renderer::image::ImageRenderer;
use crate::renderer::layout::percent_of;
use crate::renderer::theme::{ERROR_COLOR, SUCCESS_COLOR, WARNING_COLOR, default_block, dim_style};
use crate::states::AppState;
use crate::states::game::{FightPhase, Sprite};
use client_api::ApiRequest;
use client_api::commands::FightAttackCommand;
use client_api::events::FightStartData;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mpsc::Sender;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Paragraph};
use ratatui_code_editor::actions::{
    Delete, Indent, InsertNewline, InsertText, MoveDown, MoveLeft, MoveRight, MoveUp, Redo,
    UnIndent, Undo,
};
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
const SUBMIT_WIDTH: u16 = 12;
const COUNTER_WIDTH: u16 = 14;

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditorMode {
    Normal,
    Insert,
}

pub struct EditorView {
    editor: Editor,
    npc_id: String,
    time: u64,
    max_code_size: u32,
    nl_sep: String,
    sp_sep: String,
    started_at: Instant,
    editor_area: Rect,
    timed_out: bool,
    image_renderer: ImageRenderer,
    submit_button: Button,
    mode: EditorMode,
    pending: Option<char>,
    register: Option<String>,
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
            max_code_size: fight_data.max_code_size,
            nl_sep: fight_data.nl_sep.clone(),
            sp_sep: fight_data.sp_sep.clone(),
            started_at: Instant::now(),
            editor_area: Rect::default(),
            timed_out: false,
            image_renderer: ImageRenderer::new(),
            submit_button: Button::new("SUBMIT"),
            mode: EditorMode::Insert,
            pending: None,
            register: None,
        })
    }

    fn move_to_line(&mut self, row: usize) {
        let cursor = self.editor.code_ref().line_to_char(row);

        self.editor.set_cursor(cursor);
    }

    fn move_to_line_start(&mut self) {
        let code = self.editor.code_ref();
        let (row, _) = code.point(self.editor.get_cursor());
        let cursor = code.line_to_char(row);

        self.editor.set_cursor(cursor);
    }

    fn move_to_line_end(&mut self) {
        let code = self.editor.code_ref();
        let (row, _) = code.point(self.editor.get_cursor());
        let cursor = code.line_to_char(row) + code.line_len(row);

        self.editor.set_cursor(cursor);
    }

    fn move_to_last_line(&mut self) {
        let row = self.editor.code_ref().len_lines().saturating_sub(1);

        self.move_to_line(row);
    }

    fn move_to_next_word(&mut self) {
        let code = self.editor.code_ref();
        let (row, mut col) = code.point(self.editor.get_cursor());
        let length = code.line_len(row);
        let characters: Vec<char> = code.line(row).chars().collect();

        while col < length && !characters[col].is_whitespace() {
            col += 1;
        }

        while col < length && characters[col].is_whitespace() {
            col += 1;
        }

        let cursor = code.line_to_char(row) + col;

        self.editor.set_cursor(cursor);
    }

    fn move_to_previous_word(&mut self) {
        let code = self.editor.code_ref();
        let (row, mut col) = code.point(self.editor.get_cursor());
        let characters: Vec<char> = code.line(row).chars().collect();

        while col > 0 && characters[col - 1].is_whitespace() {
            col -= 1;
        }

        while col > 0 && !characters[col - 1].is_whitespace() {
            col -= 1;
        }

        let cursor = code.line_to_char(row) + col;

        self.editor.set_cursor(cursor);
    }

    fn cut_line(&mut self) {
        let code = self.editor.code_ref();
        let (row, _) = code.point(self.editor.get_cursor());
        let start = code.line_to_char(row);

        let end = if row + 1 < code.len_lines() {
            code.line_to_char(row + 1)
        } else {
            code.len_chars()
        };

        self.register = Some(code.slice(start, end));

        self.editor.set_cursor(start);
        self.editor.extend_selection(end);
        self.editor.apply(Delete {});
    }

    fn remaining_length(&self) -> usize {
        self.max_code_size
            .saturating_sub(self.editor.code_ref().len_chars() as u32) as usize
    }

    fn paste_register(&mut self) {
        let Some(register) = self.register.clone() else {
            return;
        };

        let text: String = register.chars().take(self.remaining_length()).collect();

        if text.is_empty() {
            return;
        }

        self.editor.apply(InsertText { text });
    }

    fn insert_line_below(&mut self) {
        if self.remaining_length() == 0 {
            return;
        }

        self.move_to_line_end();
        self.editor.apply(InsertNewline {});
    }

    fn insert_line_above(&mut self) {
        if self.remaining_length() == 0 {
            return;
        }

        self.move_to_line_start();
        self.editor.apply(InsertNewline {});
        self.editor.apply(MoveUp { shift: false });
    }

    fn handle_normal_key(&mut self, key: &KeyEvent) {
        if let Some(pending) = self.pending.take() {
            match (pending, key.code) {
                ('d', KeyCode::Char('d')) => self.cut_line(),
                ('g', KeyCode::Char('g')) => self.move_to_line(0),
                _ => {}
            }

            return;
        }

        match key.code {
            KeyCode::Char('h') | KeyCode::Left => self.editor.apply(MoveLeft { shift: false }),
            KeyCode::Char('j') | KeyCode::Down => self.editor.apply(MoveDown { shift: false }),
            KeyCode::Char('k') | KeyCode::Up => self.editor.apply(MoveUp { shift: false }),
            KeyCode::Char('l') | KeyCode::Right => self.editor.apply(MoveRight { shift: false }),

            KeyCode::Char('0') => self.move_to_line_start(),
            KeyCode::Char('$') => self.move_to_line_end(),
            KeyCode::Char('G') => self.move_to_last_line(),
            KeyCode::Char('w') => self.move_to_next_word(),
            KeyCode::Char('b') => self.move_to_previous_word(),

            KeyCode::Char('x') => {
                self.editor.apply(MoveRight { shift: false });
                self.editor.apply(Delete {});
            }
            KeyCode::Char('>') | KeyCode::Char('|') => self.editor.apply(Indent {}),
            KeyCode::Char('<') | KeyCode::Char('\\') => self.editor.apply(UnIndent {}),
            KeyCode::Char('u') => self.editor.apply(Undo {}),
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.apply(Redo {})
            }
            KeyCode::Char('p') => self.paste_register(),

            KeyCode::Char(character @ ('d' | 'g')) => self.pending = Some(character),

            KeyCode::Char('i') => self.mode = EditorMode::Insert,
            KeyCode::Char('a') => {
                self.editor.apply(MoveRight { shift: false });
                self.mode = EditorMode::Insert;
            }
            KeyCode::Char('I') => {
                self.move_to_line_start();
                self.mode = EditorMode::Insert;
            }
            KeyCode::Char('A') => {
                self.move_to_line_end();
                self.mode = EditorMode::Insert;
            }
            KeyCode::Char('o') => {
                self.insert_line_below();
                self.mode = EditorMode::Insert;
            }
            KeyCode::Char('O') => {
                self.insert_line_above();
                self.mode = EditorMode::Insert;
            }
            _ => {}
        }
    }

    fn submit(&self, state: &mut AppState, event_sender: &Sender<ApplicationEvent>) {
        let request = ApiRequest::FightAttack(FightAttackCommand {
            code: self.serialize_code(),
        });

        let _ = event_sender.try_send(ApplicationEvent::Send(SendEvent::ApiRequest(request)));
        state.game.fight.submit();
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
                self.image_renderer.draw_fitted(
                    frame,
                    area,
                    &state.game.assets,
                    image_path,
                    Resize::Scale(None),
                );
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

        let filled_area = Rect {
            width: percent_of(area.width, health.percent()),
            ..area
        };

        frame.render_widget(
            Block::default().style(Style::default().bg(ERROR_COLOR)),
            filled_area,
        );

        frame.render_widget(
            Paragraph::new(format!("{} / {}", health.current, health.max))
                .alignment(Alignment::Center),
            area,
        );
    }

    fn header(&self, state: &AppState) -> Paragraph<'static> {
        let npc_name = state.game.manifest.npc_name(&self.npc_id);
        let remaining = self.remaining_seconds();

        let timer_color = if remaining == 0 {
            ERROR_COLOR
        } else {
            WARNING_COLOR
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

    fn draw_footer(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let is_editing = state.game.fight.phase() == FightPhase::Editing;
        let submit_width = if is_editing { SUBMIT_WIDTH } else { 0 };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(COUNTER_WIDTH),
                Constraint::Min(1),
                Constraint::Length(submit_width),
            ])
            .split(area);

        frame.render_widget(self.counter(), chunks[0]);
        frame.render_widget(self.footer(state), chunks[1]);

        if is_editing {
            self.submit_button.draw(state, frame, chunks[2]);
        } else {
            self.submit_button.hide();
        }
    }

    fn counter(&self) -> Paragraph<'static> {
        let length = self.editor.code_ref().len_chars();

        let style = match length >= self.max_code_size as usize {
            true => Style::default()
                .fg(ERROR_COLOR)
                .add_modifier(Modifier::BOLD),
            false => dim_style(),
        };

        Paragraph::new(Span::styled(
            format!("{}/{}", length, self.max_code_size),
            style,
        ))
        .alignment(Alignment::Center)
        .block(default_block())
    }

    fn footer(&self, state: &AppState) -> Paragraph<'static> {
        let (text, style) = match state.game.fight.phase() {
            FightPhase::Editing => (
                match self.mode {
                    EditorMode::Normal => {
                        "NORMAL  hjkl 0 $ w b gg G  ·  i a I A o O  ·  x dd < > u Ctrl+R p  ·  Ctrl+S submit"
                    }
                    EditorMode::Insert => "INSERT  ·  Esc  ·  Ctrl+S submit",
                },
                dim_style(),
            ),
            FightPhase::AwaitingResult => (
                "Code submitted. Waiting for the other players...",
                dim_style(),
            ),
            FightPhase::Resolved { success: false } => (
                "Your code failed. Waiting for the other players...",
                Style::default().fg(ERROR_COLOR),
            ),
            FightPhase::Resolved { success: true } => (
                "Your code succeeded. Waiting for the other players...",
                Style::default().fg(SUCCESS_COLOR),
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
        self.draw_footer(state, frame, chunks[2]);

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
        let _ = sender.try_send(ApplicationEvent::Custom(CustomEvent::FightTimedOut));
    }

    fn on_click(
        &mut self,
        state: &mut AppState,
        column: u16,
        row: u16,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if state.game.fight.phase() != FightPhase::Editing {
            return EventFlow::Ignored;
        }

        if self.submit_button.hit(column, row) {
            self.submit(state, sender);

            return EventFlow::Consumed;
        }

        EventFlow::Ignored
    }

    fn on_key(
        &mut self,
        state: &mut AppState,
        key: &KeyEvent,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if state.game.fight.phase() != FightPhase::Editing {
            return EventFlow::Ignored;
        }

        if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.submit(state, sender);

            return EventFlow::Consumed;
        }

        match self.mode {
            EditorMode::Insert => {
                let inserts_text =
                    matches!(key.code, KeyCode::Char(_) | KeyCode::Enter | KeyCode::Tab);

                if key.code == KeyCode::Esc {
                    self.mode = EditorMode::Normal;
                } else if !inserts_text || self.remaining_length() > 0 {
                    let _ = self.editor.input(*key, &self.editor_area);
                }
            }
            EditorMode::Normal => {
                self.handle_normal_key(key);
                self.editor.focus(&self.editor_area);
            }
        }

        EventFlow::Consumed
    }
}
