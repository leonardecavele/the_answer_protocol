use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::{Direction, GameFocus, Sprite};
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::utils::{center_area_with_aspect_ratio, wrap_str_to_lines};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Clear, Paragraph},
};
use std::time::Instant;
use tokio::sync::mpsc::Sender;

const NO_ROOM: &str = " You are lost and have your eyes closed. ";
const NO_ROOM_IMAGE: &str = " No image available for this room. ";
const DISCONNECTED: &str = " Waiting for game server reconnection.. ";

const FOCUS_BADGE: &str = " [ FOCUS ] ";
const MESSAGE_MARGIN_X: u16 = 8;
const MESSAGE_MARGIN_TOP: u16 = 1;

const SCREEN_EXITS: [(&str, Placement); 4] = [
    (" [Top] ", Placement::Top),
    (" [Right] ", Placement::Right),
    (" [Down] ", Placement::Bottom),
    (" [Left] ", Placement::Left),
];

enum Content {
    Disconnected,
    Image(String),
    Message(&'static str),
}

#[derive(Clone, Copy)]
enum Placement {
    Top,
    Bottom,
    Left,
    Right,
}

impl Placement {
    fn area(&self, image_area: Rect, width: u16) -> Rect {
        let centered_x = image_area.x + image_area.width.saturating_sub(width) / 2;
        let middle_y = image_area.y + image_area.height / 2;

        let (x, y) = match self {
            Placement::Top => (centered_x, image_area.y),
            Placement::Bottom => (
                centered_x,
                image_area.y + image_area.height.saturating_sub(1),
            ),
            Placement::Left => (image_area.x, middle_y),
            Placement::Right => (
                image_area.x + image_area.width.saturating_sub(width),
                middle_y,
            ),
        };

        Rect {
            x,
            y,
            width,
            height: 1,
        }
    }
}

pub struct RightPanel {
    animation_start: Instant,
    shown_entity: Option<String>,
}

impl RightPanel {
    pub fn new() -> Self {
        Self {
            animation_start: Instant::now(),
            shown_entity: None,
        }
    }

    fn room_facing(&self, state: &AppState) -> Direction {
        match &state.game.room.id {
            Some(room_id) => Direction::facing_of_room(room_id, &state.game.manifest),
            None => Direction::default(),
        }
    }

    fn content(&self, state: &AppState) -> Content {
        if !state.network.is_connected {
            return Content::Disconnected;
        }

        let elapsed = self.animation_start.elapsed();
        let manifest = &state.game.manifest;

        if let Some(id) = &state.game.overlays.inspected_entity
            && let Some(image_path) = Sprite::of_entity(id, manifest).frame_at(elapsed)
        {
            return Content::Image(image_path.to_string());
        }

        match &state.game.room.id {
            Some(room_id) => match Sprite::of_room(room_id, manifest).frame_at(elapsed) {
                Some(image_path) => Content::Image(image_path.to_string()),
                None => Content::Message(NO_ROOM_IMAGE),
            },
            None => Content::Message(NO_ROOM),
        }
    }

    fn sync_animation(&mut self, state: &AppState) {
        let current = state.game.overlays.inspected_entity.as_deref();
        if self.shown_entity.as_deref() != current {
            self.shown_entity = current.map(str::to_owned);
            self.animation_start = Instant::now();
        }
    }

    pub fn get_desired_width(&self, state: &AppState, available_height: u16) -> Option<u16> {
        let Content::Image(image_path) = self.content(state) else {
            return None;
        };

        let (image_width, image_height) = state.ui.image_manager.get_dimensions(&image_path)?;
        let aspect = (image_width as f32) / (image_height as f32 / 2.0);

        Some((available_height as f32 * aspect) as u16)
    }

    fn draw_image(
        &self,
        state: &AppState,
        frame: &mut Frame,
        area: Rect,
        image_path: &str,
    ) -> Rect {
        let image_area = match state.ui.image_manager.get_dimensions(image_path) {
            Some((image_width, image_height)) => {
                center_area_with_aspect_ratio(area, image_width, image_height)
            }
            None => area,
        };

        state.ui.image_manager.render(
            frame,
            image_area,
            image_path,
            ratatui_image::Resize::Scale(None),
        );

        image_area
    }

    fn draw_message(&self, frame: &mut Frame, area: Rect, text: &str, color: Color) {
        let mut message_area = area;

        if message_area.height > 2 {
            message_area.y += MESSAGE_MARGIN_TOP;
            message_area.height -= MESSAGE_MARGIN_TOP;
        }
        if message_area.width > MESSAGE_MARGIN_X * 2 {
            message_area.x += MESSAGE_MARGIN_X;
            message_area.width -= MESSAGE_MARGIN_X * 2;
        }

        let lines = wrap_str_to_lines(text, message_area.width as usize);
        let lines_count = lines.len() as u16;

        if message_area.height > lines_count {
            message_area.y += message_area.height.saturating_sub(lines_count) / 2;
            message_area.height = lines_count;
        }

        frame.render_widget(
            Paragraph::new(lines).alignment(Alignment::Center).fg(color),
            message_area,
        );
    }

    fn draw_disconnected(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Red)),
            area,
        );

        self.draw_message(frame, area, DISCONNECTED, Color::Red);
    }

    fn draw_focus_badge(&self, frame: &mut Frame, area: Rect) {
        let width = FOCUS_BADGE.len() as u16;
        let badge_area = Rect {
            x: area.x + area.width.saturating_sub(width),
            y: area.y,
            width,
            height: 1,
        };

        frame.render_widget(Clear, badge_area);
        frame.render_widget(
            Paragraph::new(FOCUS_BADGE).style(Style::default().fg(Color::Yellow)),
            badge_area,
        );
    }

    fn draw_exits(&self, state: &AppState, frame: &mut Frame, image_area: Rect) {
        let style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        let facing = self.room_facing(state);

        for direction in Direction::CLOCKWISE {
            let Some(exit_name) = state.game.room.exits.get(direction.key()) else {
                continue;
            };

            let slot = (direction.quarter_turns() + 4 - facing.quarter_turns()) % 4;
            let (fallback, placement) = SCREEN_EXITS[slot];

            let label = Self::exit_label(exit_name, fallback, image_area.width);
            let exit_area = placement.area(image_area, label.chars().count() as u16);

            frame.render_widget(Clear, exit_area);
            frame.render_widget(Paragraph::new(label).style(style), exit_area);
        }
    }

    fn exit_label(exit_name: &str, fallback: &'static str, max_width: u16) -> String {
        let name = exit_name.trim();
        if name.is_empty() {
            return fallback.to_string();
        }

        let framed = format!(" [{}] ", name);
        if framed.chars().count() as u16 <= max_width / 2 {
            framed
        } else {
            fallback.to_string()
        }
    }
}

impl Component for RightPanel {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        self.sync_animation(state);

        let image_area = match self.content(state) {
            Content::Disconnected => {
                self.draw_disconnected(frame, area);
                return;
            }
            Content::Image(image_path) => self.draw_image(state, frame, area, &image_path),
            Content::Message(text) => {
                self.draw_message(frame, area, text, Color::Reset);
                area
            }
        };

        if state
            .game
            .group
            .allows_move_by(state.game.player.name.as_deref())
            && state.game.focus == GameFocus::RightPanel
        {
            self.draw_focus_badge(frame, area);
            self.draw_exits(state, frame, image_area);
        }
    }
}

impl Lifecycle for RightPanel {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if !state
            .game
            .group
            .allows_move_by(state.game.player.name.as_deref())
            || state.game.focus != GameFocus::RightPanel
        {
            return false;
        }

        let CrosstermEvent::Key(key) = event else {
            return false;
        };

        if key.code == KeyCode::Enter {
            state.game.focus = GameFocus::NpcList;
            return true;
        }

        let slot = match key.code {
            KeyCode::Up => 0,
            KeyCode::Right => 1,
            KeyCode::Down => 2,
            KeyCode::Left => 3,
            _ => return false,
        };

        let facing = self.room_facing(state);
        let direction = Direction::from_quarter_turns(slot + facing.quarter_turns());

        if !state.game.room.has_exit(direction.key()) {
            return false;
        }

        let _ = event_sender.try_send(ApplicationEvent::SendRawCommand(format!(
            "MOVE {}",
            direction.key()
        )));

        true
    }
}
