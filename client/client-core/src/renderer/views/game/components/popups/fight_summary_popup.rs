use crate::collections::Step;
use crate::events::ApplicationEvent;
use crate::renderer::components::{
    Component, EventFlow, Lifecycle, ScrollOffset, hit_row, is_mouse_in_rect,
};
use crate::renderer::layout::{centered_rect, percent_of};
use crate::renderer::text::wrap_str_to_lines;
use crate::renderer::theme::{
    ERROR_COLOR, MUTED_COLOR, SUCCESS_COLOR, close_hint, dim_style, popup_block, selection_style,
};
use crate::states::AppState;
use crate::states::game::FightSummaryState;
use client_api::events::{FightEndData, FightEndPlayerData};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph},
};
use tokio::sync::mpsc::Sender;

const POPUP_WIDTH_PERCENT: u16 = 90;
const POPUP_HEIGHT_PERCENT: u16 = 80;
const MIN_WIDTH: u16 = 40;
const MIN_HEIGHT: u16 = 10;
const LIST_WIDTH: u16 = 13;
const FOOTER_HEIGHT: u16 = 2;
const EMPTY_HISTORY: &str = " No fight has ended yet. ";

pub struct FightSummaryPopup {
    area: Option<Rect>,
    list_area: Option<Rect>,
    detail_area: Option<Rect>,
    scroll: ScrollOffset,
    shown_fight: Option<usize>,
}

impl Default for FightSummaryPopup {
    fn default() -> Self {
        Self::new()
    }
}

impl FightSummaryPopup {
    pub fn new() -> Self {
        Self {
            area: None,
            list_area: None,
            detail_area: None,
            scroll: ScrollOffset::new(),
            shown_fight: None,
        }
    }

    pub fn hit_list(&self, state: &AppState, column: u16, row: u16) -> Option<usize> {
        let row_index = hit_row(self.list_area, column, row)?;
        let count = state.game.fight.history().len();

        count.checked_sub(1)?.checked_sub(row_index)
    }

    fn local_player_outcome(state: &AppState, fight: &FightEndData) -> Option<bool> {
        fight
            .players
            .iter()
            .find(|player| state.game.player.is_me(&player.name))
            .map(|player| player.success)
    }

    fn duration_label(elapsed_ms: u32) -> String {
        let seconds = elapsed_ms / 1000;

        if seconds < 60 {
            return format!("{}s", seconds);
        }

        format!("{}m {:02}s", seconds / 60, seconds % 60)
    }

    fn sync_selection(&mut self, selected: usize) {
        if self.shown_fight != Some(selected) {
            self.shown_fight = Some(selected);
            self.scroll.reset();
        }
    }

    fn draw_list(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.list_area = Some(area);

        let Some(fight_summary_state) = state.game.overlays.get::<FightSummaryState>() else {
            return;
        };

        let items: Vec<ListItem> = state
            .game
            .fight
            .history()
            .iter()
            .enumerate()
            .rev()
            .map(|(index, fight)| {
                let color = match Self::local_player_outcome(state, fight) {
                    Some(true) => SUCCESS_COLOR,
                    Some(false) => ERROR_COLOR,
                    None => Color::Reset,
                };

                let style = selection_style(color, fight_summary_state.selected == index);

                ListItem::new(Span::styled(format!(" Fight #{}", index + 1), style))
            })
            .collect();

        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(dim_style());

        frame.render_widget(List::new(items).block(block), area);
    }

    fn player_lines(
        player: &FightEndPlayerData,
        fight: &FightEndData,
        max_width: usize,
    ) -> Vec<Line<'static>> {
        let (label, color) = match player.success {
            true => ("won", SUCCESS_COLOR),
            false => ("lost", ERROR_COLOR),
        };

        let verb = if player.success { "deals" } else { "takes" };
        let message = format!("  {verb} {} damage", player.damage_dealt);

        let mut lines = vec![Line::from(vec![
            Span::styled(
                player.name.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {}", label), Style::default().fg(color)),
            Span::styled(message, Style::default().fg(color)),
            Span::styled(
                format!("  {}", Self::duration_label(player.elapsed_ms)),
                Style::default().fg(MUTED_COLOR),
            ),
        ])];

        let code = player
            .code
            .replace(&fight.nl_sep, "\n")
            .replace(&fight.sp_sep, " ");

        for code_line in code.lines() {
            lines.extend(wrap_str_to_lines(code_line, max_width));
        }

        lines.push(Line::from(""));

        lines
    }

    fn draw_detail(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.detail_area = Some(area);

        let Some(fight_summary_state) = state.game.overlays.get::<FightSummaryState>() else {
            return;
        };

        self.sync_selection(fight_summary_state.selected);

        let Some(fight) = state.game.fight.history().get(fight_summary_state.selected) else {
            return;
        };

        let block = Block::default().padding(Padding::horizontal(1));
        let inner_area = block.inner(area);

        let lines: Vec<Line> = fight
            .players
            .iter()
            .flat_map(|player| Self::player_lines(player, fight, inner_area.width as usize))
            .collect();

        self.scroll.fit(lines.len() as u16, inner_area.height);

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .scroll((self.scroll.row(), 0)),
            area,
        );
    }
}

impl Component for FightSummaryPopup {
    fn drawn_area(&self) -> Option<Rect> {
        self.area
    }

    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        if !state.game.overlays.is_open::<FightSummaryState>() {
            return;
        }

        let popup_width = percent_of(area.width, POPUP_WIDTH_PERCENT).max(MIN_WIDTH);
        let popup_height = percent_of(area.height, POPUP_HEIGHT_PERCENT).max(MIN_HEIGHT);
        let popup_area = centered_rect(area, popup_width, popup_height);

        self.area = Some(popup_area);

        frame.render_widget(Clear, popup_area);

        let block = popup_block(" Fight history ");
        let inner_area = block.inner(popup_area);

        frame.render_widget(block, popup_area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(FOOTER_HEIGHT)])
            .split(inner_area);

        if state.game.fight.history().is_empty() {
            self.list_area = None;
            self.detail_area = None;

            frame.render_widget(Paragraph::new(EMPTY_HISTORY).style(dim_style()), rows[0]);
            frame.render_widget(close_hint(), rows[1]);
            return;
        }

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(LIST_WIDTH), Constraint::Min(1)])
            .split(rows[0]);

        self.draw_list(state, frame, columns[0]);
        self.draw_detail(state, frame, columns[1]);

        frame.render_widget(close_hint(), rows[1]);
    }
}

impl Lifecycle for FightSummaryPopup {
    fn on_key(
        &mut self,
        state: &mut AppState,
        key: &KeyEvent,
        _sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if !state.game.overlays.is_open::<FightSummaryState>() {
            return EventFlow::Ignored;
        }

        let count = state.game.fight.history().len();

        match key.code {
            KeyCode::Up | KeyCode::Down => {
                let step = match key.code {
                    KeyCode::Up => Step::Next,
                    _ => Step::Previous,
                };

                if let Some(fight_summary_state) =
                    state.game.overlays.get_mut::<FightSummaryState>()
                {
                    fight_summary_state.move_selection(step, count);
                }

                EventFlow::Consumed
            }
            KeyCode::Esc | KeyCode::Enter => {
                state.game.close_top_overlay();
                EventFlow::Consumed
            }
            _ => EventFlow::Ignored,
        }
    }

    fn on_click(
        &mut self,
        state: &mut AppState,
        column: u16,
        row: u16,
        _sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if !state.game.overlays.is_open::<FightSummaryState>() {
            return EventFlow::Ignored;
        }

        let Some(index) = self.hit_list(state, column, row) else {
            return EventFlow::Ignored;
        };

        if let Some(fight_summary_state) = state.game.overlays.get_mut::<FightSummaryState>() {
            fight_summary_state.selected = index;
        }

        EventFlow::Consumed
    }

    fn on_scroll(&mut self, state: &mut AppState, step: Step, column: u16, row: u16) -> EventFlow {
        if !state.game.overlays.is_open::<FightSummaryState>() {
            return EventFlow::Ignored;
        }

        if self
            .detail_area
            .is_some_and(|area| is_mouse_in_rect(column, row, area))
        {
            self.scroll.scroll(step, 1);
            return EventFlow::Consumed;
        }

        if !self
            .list_area
            .is_some_and(|area| is_mouse_in_rect(column, row, area))
        {
            return EventFlow::Ignored;
        }

        let count = state.game.fight.history().len();
        let selection_step = match step {
            Step::Previous => Step::Next,
            Step::Next => Step::Previous,
        };

        if let Some(fight_summary_state) = state.game.overlays.get_mut::<FightSummaryState>() {
            fight_summary_state.move_selection(selection_step, count);
        }

        EventFlow::Consumed
    }
}
