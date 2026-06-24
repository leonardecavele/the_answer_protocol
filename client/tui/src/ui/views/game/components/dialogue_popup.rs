use crate::events::ApplicationEvent;
use crate::states::app::AppState;
use crate::ui::components::Component;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};
use tokio::sync::mpsc;
use std::time::Instant;

pub const CHAR_DELAY_MS: u128 = 2;

pub struct DialoguePopupComponent;

impl DialoguePopupComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for DialoguePopupComponent {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect) {
        if let Some(dialog) = &state.game.active_dialogue {
            // Place it at the bottom of the right panel, or bottom of the provided area
            let popup_height = 8;
            let width = area.width.saturating_sub(4);
            let x = area.x + 2;
            let y = area.y + area.height.saturating_sub(popup_height + 2);
            let popup_area = Rect { x, y, width, height: popup_height };

            frame.render_widget(Clear, popup_area);

            let visible_text: String = dialog.full_text.chars().take(dialog.visible_chars).collect();
            let mut display_text = visible_text;
            
            if dialog.visible_chars == dialog.full_text.len() {
                display_text.push_str("\n\n(Press Enter to continue)");
            }

            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", dialog.npc_name))
                .style(Style::default().fg(Color::Yellow));

            let paragraph = Paragraph::new(display_text).block(block);

            frame.render_widget(paragraph, popup_area);
        }
    }

    fn handle_terminal_event(
        &mut self,
        state: &mut AppState,
        event: &CrosstermEvent,
        _event_sender: &mpsc::Sender<ApplicationEvent>,
    ) -> bool {
        if let Some(dialog) = state.game.active_dialogue.clone() {
            if let CrosstermEvent::Key(key) = event {
                if key.code == KeyCode::Enter {
                    if dialog.visible_chars < dialog.full_text.len() {
                        // Skip animation
                        if let Some(ref mut d) = state.game.active_dialogue {
                            d.visible_chars = d.full_text.len();
                        }
                    } else {
                        // Close dialog
                        state.game.active_dialogue = None;
                        let should_clear = dialog.ends_dialog 
                            || state.game.dialogue_clear_mode == crate::states::game::DialogueClearMode::AlwaysClear;
                        
                        if should_clear {
                            state.game.focused_entity_id = None;
                        }
                    }
                    return true;
                } else {
                    // Block all other keys
                    return true;
                }
            }
            // Block mouse as well
            return true;
        }
        false
    }

    fn on_tick(&mut self, state: &mut AppState) {
        if let Some(ref mut dialog) = state.game.active_dialogue {
            if dialog.visible_chars < dialog.full_text.len() {
                if dialog.last_tick.elapsed().as_millis() > CHAR_DELAY_MS {
                    dialog.visible_chars += 1;
                    dialog.last_tick = Instant::now();
                }
            }
        }
    }
}
