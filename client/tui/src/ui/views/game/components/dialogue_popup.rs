use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::OverlayKind;
use crate::ui::components::Lifecycle;
use crate::ui::components::scrollable::ScrollableComponent;
use crate::ui::theme::overlay_block;
use crate::ui::utils::wrap_str_to_lines;
use api_client::ApiRequest;
use api_client::commands::TalkCommand;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use mpsc::Sender;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Padding},
};
use std::time::Instant;
use tokio::sync::mpsc;

pub const CHAR_DELAY_MS: u128 = 2;
const MAX_HEIGHT_PERCENTAGE: u16 = 40;

pub struct DialoguePopupComponent;

impl DialoguePopupComponent {
    pub fn new() -> Self {
        Self
    }
}

impl ScrollableComponent for DialoguePopupComponent {
    fn get_area(&self, state: &AppState, max_area: Rect) -> Rect {
        let width = max_area.width.saturating_sub(4);
        let x = max_area.x + 2;

        let inner_width = width.saturating_sub(4).max(1);
        let visual_lines = self.get_content(state, inner_width as usize);

        let content_height = visual_lines.len() as u16;
        let total_needed_height = content_height + 4; // 2 borders, 2 padding

        let min_height = 6;
        let max_height = max_area.height * MAX_HEIGHT_PERCENTAGE / 100;
        let popup_height = total_needed_height.clamp(min_height, max_height);

        let y = max_area.y + max_area.height.saturating_sub(popup_height);
        Rect {
            x,
            y,
            width,
            height: popup_height,
        }
    }

    fn get_block<'a>(&self, state: &AppState) -> Block<'a> {
        if let Some(dialog) = state.game.ui.dialogue() {
            overlay_block()
                .title(format!(" {} ", dialog.npc_name))
                .style(Style::default().fg(Color::Yellow))
                .padding(Padding::uniform(1))
        } else {
            overlay_block()
        }
    }

    fn get_content<'a>(&self, state: &'a AppState, max_width: usize) -> Vec<Line<'a>> {
        if let Some(dialog) = state.game.ui.dialogue() {
            let visible_text: String = dialog
                .full_text
                .chars()
                .take(dialog.visible_chars)
                .collect();
            let mut display_text = visible_text;

            if dialog.visible_chars >= dialog.full_text.chars().count() {
                let text = if dialog.ends_dialog {
                    "(Press Enter to close)"
                } else {
                    "(Press Enter to continue)"
                };
                display_text.push_str(format!("\n\n{text}").as_str());
            }

            wrap_str_to_lines(&display_text, max_width)
        } else {
            Vec::new()
        }
    }
}

impl Lifecycle for DialoguePopupComponent {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        sender: &Sender<ApplicationEvent>,
    ) -> bool {
        if let Some(dialog) = state.game.ui.dialogue().cloned() {
            if let CrosstermEvent::Key(key) = event {
                if key.code == KeyCode::Enter {
                    if dialog.visible_chars < dialog.full_text.chars().count() {
                        if let Some(d) = state.game.ui.dialogue_mut() {
                            d.visible_chars = d.full_text.chars().count();
                        }
                    } else {
                        if dialog.ends_dialog {
                            state.game.ui.close(OverlayKind::Dialogue);
                        } else {
                            let request = ApiRequest::Talk(TalkCommand {
                                npc_name: dialog.npc_id.clone(),
                            });

                            let _ = sender.try_send(ApplicationEvent::SendRequest(request));
                        }
                    }
                    return true;
                }
            }

            return true;
        }
        false
    }

    fn on_tick(&mut self, state: &mut AppState) {
        if let Some(dialog) = state.game.ui.dialogue_mut() {
            if dialog.visible_chars < dialog.full_text.chars().count() {
                if dialog.last_tick.elapsed().as_millis() > CHAR_DELAY_MS {
                    dialog.visible_chars += 1;
                    dialog.last_tick = Instant::now();
                }
            }
        }
    }
}
