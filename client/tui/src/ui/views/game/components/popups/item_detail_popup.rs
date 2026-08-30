use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::ItemDetailState;
use crate::ui::components::{Component, EventFlow, Lifecycle};
use crate::ui::image::ImageRenderer;
use crate::ui::layout::centered_rect_percent;
use crate::ui::text::wrap_str_to_lines;
use crate::ui::theme::{close_hint, popup_block};
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Clear, Paragraph},
};
use ratatui_image::Resize;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

const POPUP_WIDTH_PERCENT: u16 = 60;
const POPUP_HEIGHT_PERCENT: u16 = 70;
const IMAGE_MIN_HEIGHT: u16 = 10;
const SPACER_HEIGHT: u16 = 1;
const DESC_HEIGHT: u16 = 6;
const FOOTER_HEIGHT: u16 = 2;

#[derive(Default)]
pub struct ItemDetailPopup {
    image_renderer: ImageRenderer,
}

impl ItemDetailPopup {
    pub fn new() -> Self {
        Self {
            image_renderer: ImageRenderer::new(),
        }
    }
}

impl Component for ItemDetailPopup {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        let item_id = match state.game.overlays.get::<ItemDetailState>() {
            Some(overlay) => overlay.item_id.as_str(),
            None => return,
        };

        let Some(item) = state.game.find_item(item_id) else {
            return;
        };

        let popup_area = centered_rect_percent(area, POPUP_WIDTH_PERCENT, POPUP_HEIGHT_PERCENT);

        frame.render_widget(Clear, popup_area);

        let block = popup_block(format!(" {} ", item.name));

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

        let image_area = chunks[0];
        let desc_area = chunks[2];
        let footer_area = chunks[3];

        match item.sprite.frame_at(Duration::ZERO) {
            Some(image_path) => {
                self.image_renderer
                    .draw_fitted(frame, image_area, image_path, Resize::Fit(None));
            }
            None => {
                let mut centered_fallback_area = image_area;
                if centered_fallback_area.height > 1 {
                    centered_fallback_area.y += centered_fallback_area.height / 2;
                    centered_fallback_area.height = 1;
                }
                let p = Paragraph::new(" No image available. ").alignment(Alignment::Center);
                frame.render_widget(p, centered_fallback_area);
            }
        }

        let desc_lines = wrap_str_to_lines(&item.description, desc_area.width as usize);
        let desc_paragraph = Paragraph::new(desc_lines).alignment(Alignment::Center);
        frame.render_widget(desc_paragraph, desc_area);

        frame.render_widget(close_hint(), footer_area);
    }
}

impl Lifecycle for ItemDetailPopup {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        if !state.game.overlays.is_open::<ItemDetailState>() {
            return EventFlow::Ignored;
        }

        let CrosstermEvent::Key(key) = event else {
            return EventFlow::Ignored;
        };

        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                state.game.overlays.close_top();
                EventFlow::Consumed
            }
            _ => EventFlow::Ignored,
        }
    }
}
