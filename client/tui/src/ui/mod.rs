pub mod game;
pub mod login;
pub mod overlays;

use crate::app::{App, Screen};
use overlays::{draw_debug_overlay, draw_help_overlay, draw_notifications};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Login => login::draw_login_screen(f, app),
        Screen::Game => game::draw_game_screen(f, app),
    }

    if app.show_debug {
        draw_debug_overlay(f, app);
    }
    if app.show_help {
        draw_help_overlay(f);
    }

    draw_notifications(f, app);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
