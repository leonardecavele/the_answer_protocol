use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::{Component, Lifecycle};
use crate::ui::theme::overlay_block;
use crate::ui::utils::{center_area_with_aspect_ratio, wrap_str_to_lines};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Clear, Paragraph},
};
use tokio::sync::mpsc::Sender;

const POPUP_WIDTH_PERCENT: u16 = 60;
const POPUP_HEIGHT_PERCENT: u16 = 70;
const IMAGE_MIN_HEIGHT: u16 = 10;
const SPACER_HEIGHT: u16 = 1;
const DESC_HEIGHT: u16 = 6;
const FOOTER_HEIGHT: u16 = 2;

pub struct ItemViewPopupComponent {}

impl ItemViewPopupComponent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for ItemViewPopupComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let item_id = if let Some(id) = &state.game.ui.active_item_view_popup {
            id
        } else {
            return;
        };

        let manifest_item = state.game.manifest.items.get(item_id);

        let display_name = manifest_item
            .map(|i| i.name.clone())
            .unwrap_or_else(|| item_id.clone());

        let description = manifest_item
            .map(|i| i.description.clone())
            .unwrap_or_else(|| "No description available.".to_string());

        let image_path = manifest_item.and_then(|i| i.image_path.clone());

        let width = (area.width * POPUP_WIDTH_PERCENT) / 100;
        let height = (area.height * POPUP_HEIGHT_PERCENT) / 100;
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let popup_area = Rect {
            x,
            y,
            width,
            height,
        };

        frame.render_widget(Clear, popup_area);

        let block = overlay_block()
            .title(format!(" {} ", display_name))
            .style(Style::default().fg(Color::Yellow));

        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(IMAGE_MIN_HEIGHT),
                Constraint::Length(SPACER_HEIGHT),
                Constraint::Length(DESC_HEIGHT),
                Constraint::Length(FOOTER_HEIGHT),
            ])
            .split(inner_area);

        let mut actual_image_area = chunks[0];
        let desc_area = chunks[2];
        let footer_area = chunks[3];

        if let Some(path) = image_path {
            if let Some((img_width, img_height)) = state.ui.image_manager.get_dimensions(&path) {
                actual_image_area =
                    center_area_with_aspect_ratio(actual_image_area, img_width, img_height);
            }

            state.ui.image_manager.render(
                frame,
                actual_image_area,
                &path,
                ratatui_image::Resize::Fit(None),
            );
        } else {
            let mut centered_fallback_area = actual_image_area;
            if centered_fallback_area.height > 1 {
                centered_fallback_area.y += centered_fallback_area.height / 2;
                centered_fallback_area.height = 1;
            }
            let p = Paragraph::new(" No image available. ").alignment(Alignment::Center);
            frame.render_widget(p, centered_fallback_area);
        }

        let desc_lines = wrap_str_to_lines(&description, desc_area.width as usize);
        let desc_paragraph = Paragraph::new(desc_lines).alignment(Alignment::Center);
        frame.render_widget(desc_paragraph, desc_area);

        let footer_paragraph = Paragraph::new(" Press ESC or ENTER to close ")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(footer_paragraph, footer_area);
    }
}

impl Lifecycle for ItemViewPopupComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if state.game.ui.active_item_view_popup.is_none() {
            return false;
        }

        if let CrosstermEvent::Key(key) = event {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                    state.game.ui.active_item_view_popup = None;
                }
                _ => {}
            }
        }

        true
    }
}
