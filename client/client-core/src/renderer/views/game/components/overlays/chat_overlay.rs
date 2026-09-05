use crate::renderer::components::{Lifecycle, ScrollableComponent};
use crate::renderer::layout::percent_of;
use crate::renderer::text::wrap_str_to_lines;
use crate::renderer::theme::overlay_block;
use crate::states::app::AppState;
use crate::states::game::{ChatChannel, ChatSender};
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::text::Line;
use ratatui::widgets::Block;

const CHAT_WIDTH_PERCENTAGE: u16 = 80;
const CHAT_HEIGHT_PERCENTAGE: u16 = 80;

pub struct ChatOverlay;

impl Default for ChatOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatOverlay {
    pub fn new() -> Self {
        Self
    }
}

impl ScrollableComponent for ChatOverlay {
    fn get_area(&self, _state: &AppState, max_area: Rect) -> Rect {
        let chat_width = percent_of(max_area.width, CHAT_WIDTH_PERCENTAGE);
        let chat_height = percent_of(max_area.height, CHAT_HEIGHT_PERCENTAGE);
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

        for msg in &state.game.chat_log {
            let (prefix, _) = match &msg.channel {
                ChatChannel::Global => ("[GLOBAL]", Color::Yellow),
                ChatChannel::Group => ("[GROUP]", Color::LightGreen),
                ChatChannel::Room => ("[ROOM]", Color::LightCyan),
                ChatChannel::Private(_) => ("[PRIVATE]", Color::LightMagenta),
            };

            let full_text = match (&msg.channel, &msg.sender) {
                (ChatChannel::Private(other), ChatSender::Me) => {
                    let is_me = state.game.player.is_me(other.as_str());

                    if is_me {
                        format!("{} (You only): {}", prefix, msg.content)
                    } else {
                        format!("{} (You) to {}: {}", prefix, other, msg.content)
                    }
                }
                (_, ChatSender::Me) => format!("{} (You): {}", prefix, msg.content),
                (_, ChatSender::Other(from)) => format!("{} ({}): {}", prefix, from, msg.content),
            };

            visual_lines.extend(wrap_str_to_lines(&full_text, max_width));
        }

        visual_lines
    }
}

impl Lifecycle for ChatOverlay {}
