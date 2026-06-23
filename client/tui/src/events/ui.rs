use crate::app::App;
use crate::events::{AppEvent, UiEvent};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use tokio::sync::mpsc;

pub async fn handle(event: UiEvent, app: &mut App, tx: &mpsc::Sender<AppEvent>) {
    match event {
        UiEvent::TerminalEvent(evt) => {
            if let Event::Key(key) = evt {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('d') => {
                            app.state.ui.show_debug = !app.state.ui.show_debug;
                            return;
                        }
                        KeyCode::Char('h') => {
                            app.state.ui.show_help = !app.state.ui.show_help;
                            return;
                        }
                        KeyCode::Char('t') => {
                            app.state.ui.show_chat = !app.state.ui.show_chat;
                            return;
                        }
                        _ => {}
                    }
                }

                if key.code == KeyCode::Esc {
                    if app.state.ui.show_debug || app.state.ui.show_help {
                        app.state.ui.show_debug = false;
                        app.state.ui.show_help = false;
                    } else {
                        app.state.should_quit = true;
                    }
                    return;
                }
            }

            app.active_component
                .handle_event(&mut app.state, &evt, tx)
                .await;
        }
        _ => {}
    }
}