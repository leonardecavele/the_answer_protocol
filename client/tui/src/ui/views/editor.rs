use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::FightPhase;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::components::lifecycle::EventFlow;
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
use ratatui::widgets::{Block, Paragraph};
use ratatui_code_editor::editor::Editor;
use ratatui_code_editor::theme::vesper;
use std::time::Instant;
use tokio::sync::mpsc;
// TODO: ajouter un timeout si l'event FightEnd n'est jamais envoye (timeout: time + 10s)
// TODO: ajouter une barre de vie + image du mob

const EDITOR_LANGUAGE: &str = "c";
const EDITOR_BG: Color = Color::Rgb(0x16, 0x16, 0x16);
const HEADER_HEIGHT: u16 = 3;
const FOOTER_HEIGHT: u16 = 3;

pub struct EditorView {
    editor: Editor,
    npc_id: String,
    npc_hp: u64,
    npc_max_hp: u64,
    time: u64,
    nl_sep: String,
    sp_sep: String,
    started_at: Instant,
    editor_area: Rect,
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
            npc_hp: fight_data.npc_hp,
            npc_max_hp: fight_data.npc_max_hp,
            time: fight_data.time,
            nl_sep: fight_data.nl_sep.clone(),
            sp_sep: fight_data.sp_sep.clone(),
            started_at: Instant::now(),
            editor_area: Rect::default(),
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
        let (text, style) = match state.game.fight {
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

        self.editor_area = chunks[1];
        frame.render_widget(
            Block::default().style(Style::default().bg(EDITOR_BG)),
            self.editor_area,
        );
        frame.render_widget(&self.editor, self.editor_area);

        if state.game.fight == FightPhase::Editing
            && let Some((x, y)) = self.editor.get_visible_cursor(&self.editor_area)
        {
            frame.set_cursor_position(Position::new(x, y));
        }
    }
}

impl Lifecycle for EditorView {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if state.game.fight != FightPhase::Editing {
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
