use crate::states::app::AppState;
use crate::states::game::ChatChannel;
use crate::ui::components::Lifecycle;
use crate::ui::components::scrollable::ScrollableComponent;
use crate::ui::theme::overlay_block;
use crate::ui::utils::wrap_str_to_lines;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Block;

const CHAT_WIDTH_PERCENTAGE: u16 = 80;
const CHAT_HEIGHT_PERCENTAGE: u16 = 80;

pub struct ChatOverlayComponent;

impl ChatOverlayComponent {
    pub fn new() -> Self {
        Self
    }
}

impl ScrollableComponent for ChatOverlayComponent {
    fn get_area(&self, _state: &AppState, max_area: Rect) -> Rect {
        let chat_width = (max_area.width * CHAT_WIDTH_PERCENTAGE) / 100;
        let chat_height = (max_area.height * CHAT_HEIGHT_PERCENTAGE) / 100;
        Rect {
            x: max_area.x + max_area.width.saturating_sub(chat_width),
            y: max_area.y + max_area.height.saturating_sub(chat_height),
            width: chat_width,
            height: chat_height,
        }
    }

    fn get_block<'a>(&self, _state: &AppState) -> Block<'a> {
        overlay_block().title(" Chat overlay (F1 to hide) ")
    }

    fn get_content<'a>(&self, state: &'a AppState, max_width: usize) -> Vec<Line<'a>> {
        let mut visual_lines = Vec::new();

        for msg in &state.game.chat_history {
            let (prefix, _) = match &msg.channel {
                ChatChannel::Global => ("[GLOBAL] ", ratatui::style::Color::Yellow),
                ChatChannel::Group => ("[GROUP] ", ratatui::style::Color::LightGreen),
                ChatChannel::Room => ("[ROOM] ", ratatui::style::Color::LightCyan),
                ChatChannel::Private(_) => ("[PRIVATE] ", ratatui::style::Color::LightMagenta),
            };

            let full_text = format!("{}{}: {}", prefix, msg.sender, msg.content);
            visual_lines.extend(wrap_str_to_lines(&full_text, max_width));
        }

        visual_lines
    }
}

impl Lifecycle for ChatOverlayComponent {}
