use crate::app::{App, GameFocus, ACTIONS};
use crate::commands::{execute_action, handle_command};
use crate::events::AppEvent;
use crossterm::event::{Event, KeyCode, KeyEvent};
use tokio::sync::mpsc;

pub async fn handle_key(
    app: &mut App,
    key: KeyEvent,
    evt: &Event,
    tx: &mpsc::UnboundedSender<AppEvent>,
) {
    match key.code {
        KeyCode::Tab => {
            app.game_focus = match app.game_focus {
                GameFocus::Input => GameFocus::Actions,
                GameFocus::Actions => GameFocus::Input,
            };
        }
        KeyCode::Up => {
            if matches!(app.game_focus, GameFocus::Actions) {
                app.selected_action = app.selected_action.saturating_sub(1);
            } else {
                app.game_scroll_offset = app.game_scroll_offset.saturating_add(1);
            }
        }
        KeyCode::Down => {
            if matches!(app.game_focus, GameFocus::Actions) {
                if app.selected_action < ACTIONS.len() - 1 {
                    app.selected_action += 1;
                }
            } else {
                app.game_scroll_offset = app.game_scroll_offset.saturating_sub(1);
            }
        }
        KeyCode::Enter => {
            if matches!(app.game_focus, GameFocus::Actions) {
                execute_action(app, tx).await;
            } else {
                let cmd_str = app.input.value().to_string();
                app.input.reset();
                if !cmd_str.trim().is_empty() {
                    handle_command(app, cmd_str, tx.clone()).await;
                }
            }
        }
        _ => {
            if matches!(app.game_focus, GameFocus::Input) {
                tui_input::backend::crossterm::EventHandler::handle_event(&mut app.input, evt);
            }
        }
    }
}
