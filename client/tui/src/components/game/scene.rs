use crate::commands::handle_command;
use crate::components::Component;
use crate::events::AppEvent;
use crate::state::{AppState, GameFocus};
use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

pub struct SceneComponent;

#[async_trait::async_trait]
impl Component for SceneComponent {
    async fn handle_event(
        &mut self,
        state: &mut AppState,
        event: &Event,
        tx: &tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Up => {
                    handle_command(state, "move north".to_string(), tx.clone());
                }
                KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                    handle_command(state, "move south".to_string(), tx.clone());
                }
                KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Right => {
                    handle_command(state, "move east".to_string(), tx.clone());
                }
                KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Left => {
                    handle_command(state, "move west".to_string(), tx.clone());
                }
                KeyCode::PageUp => {
                    if !state.game.npcs_in_room.is_empty() {
                        let current = state.ui.selected_entity_idx.unwrap_or(0);
                        state.ui.selected_entity_idx = Some(current.saturating_sub(1));
                    }
                }
                KeyCode::PageDown => {
                    if !state.game.npcs_in_room.is_empty() {
                        let max = state.game.npcs_in_room.len().saturating_sub(1);
                        let current = state.ui.selected_entity_idx.unwrap_or(0);
                        if current < max {
                            state.ui.selected_entity_idx = Some(current + 1);
                        }
                    }
                }
                KeyCode::Enter => {
                    if state.ui.selected_entity_idx.is_some() {
                        state.ui.context_menu_open = true;
                        state.ui.context_menu_idx = 0;
                    }
                }
                _ => {}
            }
        }
    }

    fn draw(&mut self, state: &mut AppState, f: &mut Frame, area: Rect) {
        let is_focused = matches!(state.ui.game_focus, GameFocus::Scene);
        let unfocused_color = Color::DarkGray;
        let focused_color = Color::Yellow;

        let (_scene_border, scene_style) = if is_focused {
            (
                focused_color,
                Style::default()
                    .fg(focused_color)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (unfocused_color, Style::default().fg(unfocused_color))
        };

        let display_name = if state.game.current_room_name.is_empty() {
            "Unknown Location".to_string()
        } else {
            state.game.current_room_name.clone()
        };
        let scene_title = format!(" Scene: {} ", display_name);
        let scene_block = Block::default()
            .borders(Borders::ALL)
            .border_style(scene_style)
            .title(scene_title);
        let inner_scene = scene_block.inner(area);
        f.render_widget(scene_block, area);

        let check_room = if state.game.current_room.is_empty() {
            "default"
        } else {
            &state.game.current_room
        };

        if let Some(dyn_img) = state
            .assets
            .get_image_for_context(check_room, &state.game.npcs_in_room)
        {
            let img_widget = ratatui_image::StatefulImage::new().resize(ratatui_image::Resize::Scale(None));
            f.render_stateful_widget(img_widget, inner_scene, dyn_img);
        } else {
            f.render_widget(
                Paragraph::new(">> IMAGE NOT FOUND / FAILED TO LOAD <<")
                    .style(Style::default().fg(Color::Red)),
                inner_scene,
            );
        }

        if is_focused {
            let north = Paragraph::new("[N]orth")
                .style(Style::default().fg(Color::Yellow).bg(Color::Black));
            let south = Paragraph::new("[S]outh")
                .style(Style::default().fg(Color::Yellow).bg(Color::Black));
            let east =
                Paragraph::new("[E]ast").style(Style::default().fg(Color::Yellow).bg(Color::Black));
            let west =
                Paragraph::new("[W]est").style(Style::default().fg(Color::Yellow).bg(Color::Black));

            let mut n_area = inner_scene;
            n_area.height = 1;
            n_area.width = 7;
            n_area.x = inner_scene.x + (inner_scene.width / 2).saturating_sub(3);
            n_area.y = inner_scene.y;

            let mut s_area = inner_scene;
            s_area.height = 1;
            s_area.width = 7;
            s_area.x = inner_scene.x + (inner_scene.width / 2).saturating_sub(3);
            s_area.y = inner_scene.y + inner_scene.height.saturating_sub(1);

            let mut e_area = inner_scene;
            e_area.height = 1;
            e_area.width = 6;
            e_area.x = inner_scene.x + inner_scene.width.saturating_sub(6);
            e_area.y = inner_scene.y + (inner_scene.height / 2);

            let mut w_area = inner_scene;
            w_area.height = 1;
            w_area.width = 6;
            w_area.x = inner_scene.x;
            w_area.y = inner_scene.y + (inner_scene.height / 2);

            f.render_widget(north, n_area);
            f.render_widget(south, s_area);
            f.render_widget(east, e_area);
            f.render_widget(west, w_area);
        }

        if !state.game.npcs_in_room.is_empty() {
            let entities_str = state
                .game
                .npcs_in_room
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    let dname = state.assets.get_display_name(n);
                    if Some(i) == state.ui.selected_entity_idx {
                        format!("[{}]", dname)
                    } else {
                        dname
                    }
                })
                .collect::<Vec<_>>()
                .join("  ");

            let entities_p = Paragraph::new(format!(" Entities: {} ", entities_str)).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
            f.render_widget(
                entities_p,
                Rect {
                    x: inner_scene.x,
                    y: inner_scene.y + inner_scene.height.saturating_sub(1),
                    width: inner_scene.width,
                    height: 1,
                },
            );
        }
    }
}
