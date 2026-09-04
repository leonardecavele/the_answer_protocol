use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::states::game::DialogueState;
use crate::ui::components::{EventFlow, Lifecycle, ScrollableComponent};
use crate::ui::layout::percent_of;
use crate::ui::text::wrap_str_to_lines;
use crate::ui::theme::{overlay_block, popup_block};
use api_client::ApiRequest;
use api_client::commands::TalkCommand;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use mpsc::Sender;
use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Padding},
};
use std::time::Instant;
use tokio::sync::mpsc;

const CHAR_DELAY_MS: u128 = 2;
const MAX_HEIGHT_PERCENTAGE: u16 = 40;

pub struct DialoguePopup {
    chars_shown: usize,
    last_tick: Instant,
    shown_npc: Option<String>,
}

impl Default for DialoguePopup {
    fn default() -> Self {
        Self::new()
    }
}

impl DialoguePopup {
    pub fn new() -> Self {
        Self {
            chars_shown: 0,
            last_tick: Instant::now(),
            shown_npc: None,
        }
    }
}

impl ScrollableComponent for DialoguePopup {
    fn get_area(&self, state: &AppState, max_area: Rect) -> Rect {
        let width = max_area.width.saturating_sub(4);
        let x = max_area.x + 2;

        let inner_width = width.saturating_sub(4).max(1);
        let visual_lines = self.get_content(state, inner_width as usize);

        let content_height = visual_lines.len() as u16;
        let total_needed_height = content_height + 4; // 2 borders, 2 padding

        let min_height = 6;
        let max_height = percent_of(max_area.height, MAX_HEIGHT_PERCENTAGE).max(min_height);
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
        if let Some(dialog) = state.game.overlays.get::<DialogueState>() {
            popup_block(format!(" {} ", dialog.npc_name)).padding(Padding::uniform(1))
        } else {
            overlay_block()
        }
    }

    fn get_content<'a>(&self, state: &'a AppState, max_width: usize) -> Vec<Line<'a>> {
        if let Some(dialog) = state.game.overlays.get::<DialogueState>() {
            let mut display_text: String =
                dialog.full_text.chars().take(self.chars_shown).collect();

            if self.chars_shown >= dialog.char_count() {
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

impl Lifecycle for DialoguePopup {
    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        sender: &Sender<ApplicationEvent>,
    ) -> EventFlow {
        let Some(dialog) = state.game.overlays.get::<DialogueState>().cloned() else {
            return EventFlow::Ignored;
        };

        let CrosstermEvent::Key(key) = event else {
            return EventFlow::Ignored;
        };

        if key.code != KeyCode::Enter {
            return EventFlow::Ignored;
        }

        if self.chars_shown < dialog.char_count() {
            self.chars_shown = dialog.char_count();
        } else if dialog.ends_dialog {
            state.game.close_dialogue();
        } else {
            let request = ApiRequest::Talk(TalkCommand {
                npc_name: dialog.npc_id.clone(),
            });

            let _ = sender.try_send(ApplicationEvent::SendRequest(request));
        }

        EventFlow::Consumed
    }

    fn on_tick(&mut self, state: &mut AppState, _sender: &Sender<ApplicationEvent>) {
        let Some(dialog) = state.game.overlays.get::<DialogueState>() else {
            self.chars_shown = 0;
            self.shown_npc = None;
            return;
        };

        if self.shown_npc.as_deref() != Some(dialog.npc_id.as_str()) {
            self.shown_npc = Some(dialog.npc_id.clone());
            self.chars_shown = 0;
        }

        if self.chars_shown < dialog.char_count()
            && self.last_tick.elapsed().as_millis() > CHAR_DELAY_MS
        {
            self.chars_shown += 1;
            self.last_tick = Instant::now();
        }
    }
}
