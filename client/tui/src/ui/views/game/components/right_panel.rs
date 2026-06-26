use crate::states::app::AppState;
use crate::ui::components::Component;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Clear, Paragraph},
};

pub struct RightPanelComponent {
    animation_start: std::time::Instant,
    last_entity: Option<String>,
}

impl RightPanelComponent {
    pub fn new() -> Self {
        Self {
            animation_start: std::time::Instant::now(),
            last_entity: None,
        }
    }

    pub fn ensure_loaded(&mut self, state: &AppState, path: &str) {
        let mut cache = state.ui.image_cache.borrow_mut();
        if !cache.contains_key(path) {
            let start_loading = std::time::Instant::now();
            match image::open(path) {
                Ok(dyn_img) => {
                    let width = dyn_img.width();
                    let height = dyn_img.height();
                    let protocol = state.ui.image_picker.new_resize_protocol(dyn_img);
                    cache.insert(path.to_string(), Some((protocol, width, height)));
                }
                Err(_) => {
                    cache.insert(path.to_string(), None);
                }
            }
            self.animation_start -= std::time::Instant::now() - start_loading;
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
                        self.animation_start = std::time::Instant::now();
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
            self.ensure_loaded(state, &path);
            let cache = state.ui.image_cache.borrow();
            if let Some(Some((_, img_width, img_height))) = cache.get(&path) {
                let img_aspect = (*img_width as f32) / (*img_height as f32 / 2.0);
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
            self.ensure_loaded(state, &path);
            let mut cache = state.ui.image_cache.borrow_mut();

            if let Some(Some((protocol, img_width, img_height))) = cache.get_mut(&path) {
                let img_aspect = (*img_width as f32) / (*img_height as f32 / 2.0);
                let area_aspect = (inner_area.width as f32) / (inner_area.height as f32);

                if img_aspect > area_aspect {
                    let render_height = (inner_area.width as f32 / img_aspect) as u16;
                    if actual_image_area.height > render_height {
                        actual_image_area.y +=
                            (actual_image_area.height.saturating_sub(render_height)) / 2;
                        actual_image_area.height = render_height;
                    }
                } else {
                    let render_width = (inner_area.height as f32 * img_aspect) as u16;
                    if actual_image_area.width > render_width {
                        actual_image_area.x +=
                            (actual_image_area.width.saturating_sub(render_width)) / 2;
                        actual_image_area.width = render_width;
                    }
                }

                let image_widget = ratatui_image::StatefulImage::default()
                    .resize(ratatui_image::Resize::Scale(None));
                frame.render_stateful_widget(image_widget, actual_image_area, protocol);
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

                let visual_lines =
                    crate::ui::utils::wrap_str_to_lines(text, safe_area.width as usize);
                let lines_count = visual_lines.len() as u16;
                let p = Paragraph::new(visual_lines).alignment(Alignment::Center);

                if safe_area.height > lines_count {
                    safe_area.y += safe_area.height.saturating_sub(lines_count) / 2;
                    safe_area.height = lines_count;
                }
                frame.render_widget(p, safe_area);
            }
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

            let visual_lines = crate::ui::utils::wrap_str_to_lines(text, safe_area.width as usize);
            let lines_count = visual_lines.len() as u16;
            let p = Paragraph::new(visual_lines).alignment(Alignment::Center);

            if safe_area.height > lines_count {
                safe_area.y += safe_area.height.saturating_sub(lines_count) / 2;
                safe_area.height = lines_count;
            }
            frame.render_widget(p, safe_area);
        }

        if state.ui.current_focus == crate::states::ui::GameFocus::RightPanel {
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

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &crossterm::event::Event,
        event_sender: &tokio::sync::mpsc::Sender<crate::events::ApplicationEvent>,
    ) -> bool {
        if state.ui.current_focus == crate::states::ui::GameFocus::RightPanel {
            if let crossterm::event::Event::Key(key) = event {
                if key.code == crossterm::event::KeyCode::Enter {
                    state.ui.current_focus = crate::states::ui::GameFocus::NpcList;
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
                    let _ = event_sender.try_send(crate::events::ApplicationEvent::SendRawCommand(
                        format!("MOVE {}", direction),
                    ));
                    return true;
                }
            }
        }
        false
    }
}
