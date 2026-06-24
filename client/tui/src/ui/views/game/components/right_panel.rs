use crate::states::app::AppState;
use crate::states::ui::Notification;
use crate::ui::components::Component;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph},
};

pub struct RightPanelComponent;

impl RightPanelComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for RightPanelComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let mut path_to_load: Option<String> = None;
        let mut text_fallback: Option<&str> = Some(" You are lost and have your eyes closed. ");

        // Priority 1: focused entity
        if let Some(focused_id) = &state.game.focused_entity_id {
            text_fallback = Some(" No image available for this NPC. ");
            if let Some(npc) = state.game.manifest.npcs.get(focused_id) {
                if let Some(path) = &npc.image_path {
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

        // Priority 2: room
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

        let inner_area = area; // No borders

        if let Some(path) = path_to_load {
            let mut cache = state.ui.image_cache.borrow_mut();

            if !cache.contains_key(&path) {
                match image::open(&path) {
                    Ok(dyn_img) => {
                        let width = dyn_img.width();
                        let height = dyn_img.height();
                        let protocol = state.ui.image_picker.new_resize_protocol(dyn_img);
                        cache.insert(path.clone(), Some((protocol, width, height)));
                    }
                    Err(_) => {
                        cache.insert(path.clone(), None);
                    }
                }
            }

            if let Some(Some((protocol, img_width, img_height))) = cache.get_mut(&path) {
                let img_aspect = (*img_width as f32) / (*img_height as f32 / 2.0);
                let area_aspect = (inner_area.width as f32) / (inner_area.height as f32);

                let mut image_area = inner_area;

                if img_aspect > area_aspect {
                    let render_height = (inner_area.width as f32 / img_aspect) as u16;
                    if image_area.height > render_height {
                        image_area.y += (image_area.height.saturating_sub(render_height)) / 2;
                        image_area.height = render_height;
                    }
                } else {
                    let render_width = (inner_area.height as f32 * img_aspect) as u16;
                    if image_area.width > render_width {
                        image_area.x += (image_area.width.saturating_sub(render_width)) / 2;
                        image_area.width = render_width;
                    }
                }

                let image_widget = ratatui_image::StatefulImage::default().resize(ratatui_image::Resize::Scale(None));
                frame.render_stateful_widget(image_widget, image_area, protocol);
            } else if let Some(text) = text_fallback {
                let visual_lines =
                    crate::ui::utils::wrap_str_to_lines(text, inner_area.width as usize);
                let lines_count = visual_lines.len() as u16;
                let p = Paragraph::new(visual_lines).alignment(Alignment::Center);

                let mut p_area = inner_area;
                if p_area.height > lines_count {
                    p_area.y += p_area.height.saturating_sub(lines_count) / 2;
                    p_area.height = lines_count;
                }
                frame.render_widget(p, p_area);
            }
        } else if let Some(text) = text_fallback {
            let visual_lines = crate::ui::utils::wrap_str_to_lines(text, inner_area.width as usize);
            let lines_count = visual_lines.len() as u16;
            let p = Paragraph::new(visual_lines).alignment(Alignment::Center);

            let mut p_area = inner_area;
            if p_area.height > lines_count {
                p_area.y += p_area.height.saturating_sub(lines_count) / 2;
                p_area.height = lines_count;
            }
            frame.render_widget(p, p_area);
        }
    }
}
