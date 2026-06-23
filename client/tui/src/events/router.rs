use crate::app::App;
use crate::events::{net, AppEvent, GameEvent, NetEvent, UiEvent};
use crate::network;
use crate::state::NotificationType;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn route(event: AppEvent, app: &mut App, tx: &mpsc::Sender<AppEvent>) {
    match &event {
        AppEvent::Game(e) => app.state.game.handle_event(e),
        AppEvent::Network(e) => app.state.net.handle_event(e),
        AppEvent::Ui(e) => app.state.ui.handle_event(e),
    }

    match event {
        AppEvent::Game(e) => game::handle(e, app, tx).await,
        AppEvent::Network(e) => net::handle(e, app, tx).await,
        AppEvent::Ui(e) => ui::handle(e, app, tx).await,
    }

    // 2. Handle specific effects and UI dispatching
    match event {
        AppEvent::Ui(UiEvent::Tick) => {
            if let Some(client_arc) = &app.state.net.client {
                if let Ok(c) = client_arc.try_lock() {
                    if !c.is_connected() {
                        let _ = tx.send(AppEvent::Network(NetEvent::ApiDisconnected));
                    }
                }
            }
        }
        AppEvent::Ui(UiEvent::TerminalEvent(evt)) => {
            if let Event::Key(key) = evt {
                // Global Ctrl shortcuts
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

                // Esc: close overlays or quit
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

            // Dispatch to active component
            app.active_component
                .handle_event(&mut app.state, &evt, tx)
                .await;
        }
        AppEvent::Network(NetEvent::AttemptConnect(name, ip, port)) => {
            let _ = tx.send(AppEvent::Ui(UiEvent::Notification(
                format!("Connecting to {}:{}...", ip, port),
                NotificationType::Info,
                10,
            )));
            
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                use api_client::client::connect::ClientConnect;

                match ClientConnect::connect(format!("{}:{}", ip, port)).await {
                    Ok(mut client) => {
                        let tx_ev = tx_clone.clone();
                        // Setup listeners
                        client.on_event(move |ev| {
                            let _ = tx_ev.send(AppEvent::Network(NetEvent::Api(ev)));
                        });

                        let _ = tx_clone.send(AppEvent::Network(NetEvent::ClientConnected(client)));
                        let _ = tx_clone.send(AppEvent::Network(NetEvent::ExecuteConnect(name)));
                    }
                    Err(e) => {
                        log::error!("TCP Connection failed: {}", e);
                        let _ = tx_clone.send(AppEvent::Network(NetEvent::TapError(e)));
                        let _ = tx_clone.send(AppEvent::Network(NetEvent::ApiDisconnected));
                    }
                }
            });
        }
        AppEvent::Network(NetEvent::ExecuteConnect(name)) => {
            if let Some(client_arc) = &app.state.net.client {
                let client_arc = Arc::clone(client_arc);
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    let mut c = client_arc.lock().await;
                    match c.connect(name.clone()).await {
                        Ok(Ok(_)) => {
                            log::info!("Connected as {}", name);
                            let _ = tx_clone.send(AppEvent::Network(NetEvent::LoginSuccess(name)));
                        }
                        Ok(Err(err)) => {
                            log::error!("Connect failed: {:?}", err);
                            let _ = tx_clone.send(AppEvent::Game(GameEvent::CommandError(err)));
                            let _ = tx_clone.send(AppEvent::Network(NetEvent::ApiDisconnected));
                        }
                        Err(err) => {
                            log::error!("Network error: {:?}", err);
                            let _ = tx_clone.send(AppEvent::Network(NetEvent::TapError(err)));
                            let _ = tx_clone.send(AppEvent::Network(NetEvent::ApiDisconnected));
                        }
                    }
                });
            }
        }
        AppEvent::Network(NetEvent::LoginSuccess(name)) => {
            app.switch_to_game();
            let _ = tx.send(AppEvent::Game(GameEvent::PushGameOutput(format!("Welcome, {}!", name))));

            if let Some(client_arc) = &app.state.net.client {
                let client_arc = Arc::clone(client_arc);
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    let c = client_arc.lock().await;
                    if let Ok(Ok(data)) = c.who().await {
                        let _ = tx_clone.send(AppEvent::Game(GameEvent::UpdateOnlinePlayers(data.player_count)));
                    }
                });
            }
        }
        AppEvent::Network(NetEvent::NetworkError(err)) => {
            let _ = tx.send(AppEvent::Ui(UiEvent::Notification(format!("Network error: {}", err), NotificationType::Error, 16)));
        }
        AppEvent::Network(NetEvent::TapError(err)) => {
            let _ = tx.send(AppEvent::Ui(UiEvent::Notification(format!("Tap error: {}", err), NotificationType::Error, 16)));
        }
        AppEvent::Network(NetEvent::ClientConnected(client)) => {
            app.state.net.client = Some(Arc::new(tokio::sync::Mutex::new(client)));
            let _ = tx.send(AppEvent::Game(GameEvent::PushGameOutput("TCP connection established.".to_string())));
        }
        AppEvent::Network(NetEvent::ApiDisconnected) => {
            app.switch_to_login();
            let _ = tx.send(AppEvent::Ui(UiEvent::Notification("Disconnected from server".to_string(), NotificationType::Error, 16)));
        }
        AppEvent::Network(NetEvent::Api(api_evt)) => {
            network::handle_server_event(&mut app.state, api_evt);
        }
        AppEvent::Game(GameEvent::CommandError(err)) => {
            let message = match err.code {
                Some(code) => format!("[{}] Command error: {}", code, err.message),
                None => format!("Command error: {}", err.message),
            };
            let _ = tx.send(AppEvent::Ui(UiEvent::Notification(message, NotificationType::Error, 16)));
        }
        AppEvent::Game(GameEvent::UnknowCommand(err)) => {
            let _ = tx.send(AppEvent::Ui(UiEvent::Notification(format!("Unknow command: {}", err), NotificationType::Error, 16)));
        }
        _ => {} // Remaining state updates are handled purely by the reducers
    }
}
