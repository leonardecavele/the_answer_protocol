use crate::states::app::AppState;
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
        let mut title = " Cluster 6 (the backroom) ".to_string();
        let mut path_to_load: Option<String> = None;
        let mut text_fallback: Option<&str> = Some(" You are lost and have your eyes closed. ");

        // Priority 1: focused entity
        if let Some(focused_id) = &state.game.focused_entity_id {
            if let Some(npc) = state.game.manifest.npcs.get(focused_id) {
                if let Some(path) = &npc.image_path {
                    path_to_load = Some(path.clone());
                    text_fallback = None;
                }
            } else if let Some(item) = state.game.manifest.items.get(focused_id) {
                if let Some(path) = &item.image_path {
                    path_to_load = Some(path.clone());
                    text_fallback = None;
                }
            }
        }

        // Priority 2: room
        if path_to_load.is_none() {
            if let Some(room_id) = &state.game.current_room_id {
                if let Some(room) = state.game.manifest.rooms.get(room_id) {
                    title = format!(" {} ", room.name);
                    if let Some(path) = &room.image_path {
                        path_to_load = Some(path.clone());
                        text_fallback = None;
                    } else {
                        text_fallback = Some(" No image available for this room. ");
                    }
                } else {
                    title = format!(" {} ", room_id);
                    text_fallback = Some(" No image available for this room. ");
                }
            }
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title);
            
        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if let Some(path) = path_to_load {
            let mut cache = state.ui.image_cache.borrow_mut();
            
            if !cache.contains_key(&path) {
                match image::open(&path) {
                    Ok(dyn_img) => {
                        let protocol = state.ui.image_picker.new_resize_protocol(dyn_img);
                        cache.insert(path.clone(), Some(protocol));
                    }
                    Err(_) => {
                        cache.insert(path.clone(), None);
                    }
                }
            }

            if let Some(Some(protocol)) = cache.get_mut(&path) {
                let image_widget = ratatui_image::StatefulImage::default();
                frame.render_stateful_widget(image_widget, inner_area, protocol);
            } else if let Some(text) = text_fallback {
                let p = Paragraph::new(text).alignment(Alignment::Center);
                let mut p_area = inner_area;
                if p_area.height > 2 {
                    p_area.y += p_area.height / 2;
                    p_area.height = 1;
                }
                frame.render_widget(p, p_area);
            }
        } else if let Some(text) = text_fallback {
            let p = Paragraph::new(text).alignment(Alignment::Center);
            let mut p_area = inner_area;
            if p_area.height > 2 {
                p_area.y += p_area.height / 2;
                p_area.height = 1;
            }
            frame.render_widget(p, p_area);
        }
    }
}
