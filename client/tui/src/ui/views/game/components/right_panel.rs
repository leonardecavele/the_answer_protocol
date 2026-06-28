use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::ui::GameFocus;
use crate::ui::components::Component;
use crate::ui::components::Lifecycle;
use crate::ui::utils::{render_image, wrap_str_to_lines, center_area_with_aspect_ratio};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Clear, Paragraph},
};
use std::time::Instant;
use tokio::sync::mpsc::Sender;

pub struct RightPanelComponent {
    animation_start: Instant,
    last_entity: Option<String>,
}

impl RightPanelComponent {
    pub fn new() -> Self {
        Self {
            animation_start: Instant::now(),
            last_entity: None,
        }
    }

    pub fn get_path_to_load(&mut self, state: &AppState) -> (Option<String>, Option<&'static str>) {
        let mut path_to_load: Option<String> = None;
        let mut text_fallback: Option<&'static str> =
            Some(" You are lost and have your eyes closed. ");

        if let Some(focused_id) = &state.game.focused_entity_id {
            text_fallback = Some(" No image available for this NPC. ");
            if let Some(npc) = state.game.manifest.npcs.get(focused_id) {
                if let (Some(paths), Some(speed)) = (&npc.image_paths, npc.animation_speed_ms) {
                    if self.last_entity.as_deref() != Some(focused_id.as_str()) {
                        self.last_entity = Some(focused_id.clone());
                        self.animation_start = Instant::now();
                    }

                    if !paths.is_empty() {
                        let elapsed_ms = self.animation_start.elapsed().as_millis() as u64;
                        let frame_index = (elapsed_ms / speed) % paths.len() as u64;
                        path_to_load = Some(paths[frame_index as usize].clone());
                        text_fallback = Some(" Image cannot be loaded (invalid path). ");
                    }
                } else if let Some(path) = &npc.image_path {
                    path_to_load = Some(path.clone());
                    text_fallback = Some(" Image cannot be loaded (invalid path). ");
                }
            } else if let Some(item) = state.game.manifest.items.get(focused_id) {
                if let Some(path) = &item.image_path {
                    path_to_load = Some(path.clone());
                    text_fallback = Some(" Image cannot be loaded (invalid path). ");
                }
            }
        }

        if path_to_load.is_none() {
            if let Some(room_id) = &state.game.current_room_id {
                if let Some(room) = state.game.manifest.rooms.get(room_id) {
                    if let Some(path) = &room.image_path {
                        path_to_load = Some(path.clone());
                    } else {
                        text_fallback = Some(" No image available for this room. ");
                    }
                } else {
                    text_fallback = Some(" No image available for this room. ");
                }
            }
        }

        (path_to_load, text_fallback)
    }

    pub fn get_desired_width(&mut self, state: &AppState, available_height: u16) -> Option<u16> {
        let (path_to_load, _) = self.get_path_to_load(state);
        if let Some(path) = path_to_load {
            if let Some((img_width, img_height)) = state.ui.get_image_dimensions(&path) {
                let img_aspect = (img_width as f32) / (img_height as f32 / 2.0);
                let render_width = (available_height as f32 * img_aspect) as u16;
                return Some(render_width);
            }
        }
        None
    }
}

impl Component for RightPanelComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let (path_to_load, text_fallback) = self.get_path_to_load(state);

        let inner_area = area; // No borders
        let mut actual_image_area = inner_area;

        if let Some(path) = path_to_load {
            if let Some((img_width, img_height)) = state.ui.get_image_dimensions(&path) {
                actual_image_area = center_area_with_aspect_ratio(inner_area, img_width, img_height);
            }

            render_image(
                state,
                frame,
                actual_image_area,
                &path,
                ratatui_image::Resize::Scale(None),
            );
        } else if let Some(text) = text_fallback {
            let mut safe_area = inner_area;
            if safe_area.height > 2 {
                safe_area.y += 1;
                safe_area.height -= 1;
            }
            if safe_area.width > 16 {
                safe_area.x += 8;
                safe_area.width -= 16;
            }

            let visual_lines = wrap_str_to_lines(text, safe_area.width as usize);
            let lines_count = visual_lines.len() as u16;
            let p = Paragraph::new(visual_lines).alignment(Alignment::Center);

            if safe_area.height > lines_count {
                safe_area.y += safe_area.height.saturating_sub(lines_count) / 2;
                safe_area.height = lines_count;
            }
            frame.render_widget(p, safe_area);
        }

        if state.ui.current_focus == GameFocus::RightPanel {
            let focus_text = " [ FOCUS ] ";
            let focus_area = Rect {
                x: inner_area.x + inner_area.width.saturating_sub(11),
                y: inner_area.y,
                width: 11,
                height: 1,
            };
            frame.render_widget(Clear, focus_area);
            frame.render_widget(
                Paragraph::new(focus_text).style(Style::default().fg(Color::Yellow)),
                focus_area,
            );

            let exit_style = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD);

            if state.game.current_room_exits.contains_key("NORTH") {
                let text = " [North] ";
                let w = text.len() as u16;
                let x = actual_image_area.x + actual_image_area.width.saturating_sub(w) / 2;
                let area = Rect {
                    x,
                    y: actual_image_area.y,
                    width: w,
                    height: 1,
                };
                frame.render_widget(Clear, area);
                frame.render_widget(Paragraph::new(text).style(exit_style), area);
            }
            if state.game.current_room_exits.contains_key("SOUTH") {
                let text = " [South] ";
                let w = text.len() as u16;
                let x = actual_image_area.x + actual_image_area.width.saturating_sub(w) / 2;
                let y = actual_image_area.y + actual_image_area.height.saturating_sub(1);
                let area = Rect {
                    x,
                    y,
                    width: w,
                    height: 1,
                };
                frame.render_widget(Clear, area);
                frame.render_widget(Paragraph::new(text).style(exit_style), area);
            }
            if state.game.current_room_exits.contains_key("EAST") {
                let text = " [East] ";
                let w = text.len() as u16;
                let x = actual_image_area.x + actual_image_area.width.saturating_sub(w);
                let y = actual_image_area.y + actual_image_area.height / 2;
                let area = Rect {
                    x,
                    y,
                    width: w,
                    height: 1,
                };
                frame.render_widget(Clear, area);
                frame.render_widget(Paragraph::new(text).style(exit_style), area);
            }
            if state.game.current_room_exits.contains_key("WEST") {
                let text = " [West] ";
                let w = text.len() as u16;
                let x = actual_image_area.x;
                let y = actual_image_area.y + actual_image_area.height / 2;
                let area = Rect {
                    x,
                    y,
                    width: w,
                    height: 1,
                };
                frame.render_widget(Clear, area);
                frame.render_widget(Paragraph::new(text).style(exit_style), area);
            }
        }
    }
}

impl Lifecycle for RightPanelComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if state.ui.current_focus == GameFocus::RightPanel {
            if let crossterm::event::Event::Key(key) = event {
                if key.code == crossterm::event::KeyCode::Enter {
                    state.ui.current_focus = GameFocus::NpcList;
                    return true;
                }

                let direction = match key.code {
                    crossterm::event::KeyCode::Up => "NORTH",
                    crossterm::event::KeyCode::Down => "SOUTH",
                    crossterm::event::KeyCode::Right => "EAST",
                    crossterm::event::KeyCode::Left => "WEST",
                    _ => return false,
                };

                if state.game.current_room_exits.contains_key(direction) {
                    let _ = event_sender.try_send(ApplicationEvent::SendRawCommand(format!(
                        "MOVE {}",
                        direction
                    )));
                    return true;
                }
            }
        }
        false
    }
}
