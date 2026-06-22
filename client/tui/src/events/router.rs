use crate::app::App;
use crate::events::AppEvent;
use crate::network;
use crate::state::{ConnectionState, NotificationType};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn route(event: AppEvent, app: &mut App, tx: &mpsc::UnboundedSender<AppEvent>) {
    match event {
        AppEvent::Tick => {
            if let Some(client_arc) = &app.state.net.client {
                if let Ok(c) = client_arc.try_lock() {
                    if !c.is_connected() {
                        let _ = tx.send(AppEvent::ApiDisconnected);
                    }
                }
            }

            app.state.ui.notifications.retain_mut(|notif| {
                notif.lifetime = notif.lifetime.saturating_sub(1);
                notif.lifetime > 0
            });
        }
        AppEvent::TerminalEvent(evt) => {
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
        AppEvent::AttemptConnect(name, ip, port) => {
            app.state.ui.push_notification(
                format!("Connecting to {}:{}...", ip, port),
                NotificationType::Info,
                10,
            );
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                use api_client::client::connect::ClientConnect;

                match ClientConnect::connect(format!("{}:{}", ip, port)).await {
                    Ok(mut client) => {
                        let tx_ev = tx_clone.clone();
                        // Setup listeners
                        client.on_event(move |ev| {
                            let _ = tx_ev.send(AppEvent::Api(ev));
                        });

                        let _ = tx_clone.send(AppEvent::ClientConnected(client));
                        let _ = tx_clone.send(AppEvent::ExecuteConnect(name));
                    }
                    Err(e) => {
                        log::error!("TCP Connection failed: {}", e);
                        let _ = tx_clone.send(AppEvent::TapError(e));
                        let _ = tx_clone.send(AppEvent::ApiDisconnected);
                    }
                }
            });
        }
        AppEvent::ExecuteConnect(name) => {
            if let Some(client_arc) = &app.state.net.client {
                let client_arc = Arc::clone(client_arc);
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    let mut c = client_arc.lock().await;
                    match c.connect(name.clone()).await {
                        Ok(Ok(_)) => {
                            log::info!("Connected as {}", name);
                            let _ = tx_clone.send(AppEvent::LoginSuccess(name));
                        }
                        Ok(Err(err)) => {
                            log::error!("Connect failed: {:?}", err);
                            let _ = tx_clone.send(AppEvent::CommandError(err));
                            let _ = tx_clone.send(AppEvent::ApiDisconnected);
                        }
                        Err(err) => {
                            log::error!("Network error: {:?}", err);
                            let _ = tx_clone.send(AppEvent::TapError(err));
                            let _ = tx_clone.send(AppEvent::ApiDisconnected);
                        }
                    }
                });
            }
        }
        AppEvent::LoginSuccess(name) => {
            app.state.net.connection_state = ConnectionState::Connected(name.clone());
            app.state.net.connected_at = Some(std::time::Instant::now());
            app.switch_to_game();
            app.state
                .game
                .push_game_output(format!("Welcome, {}!", name));

            if let Some(client_arc) = &app.state.net.client {
                let client_arc = Arc::clone(client_arc);
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    let c = client_arc.lock().await;
                    if let Ok(Ok(data)) = c.who().await {
                        let _ =
                            tx_clone.send(AppEvent::UpdateOnlinePlayers(data.player_count));
                    }
                });
            }
        }
        AppEvent::CommandResult(res) => {
            for line in res.lines() {
                app.state.game.push_game_output(format!("{}", line));
            }
        }
        AppEvent::CommandError(err) => {
            let message = match err.code {
                Some(code) => format!("[{}] Command error: {}", code, err.message),
                None => format!("Command error: {}", err.message),
            };

            app.state.game.push_game_output(message.clone());
            app.state
                .ui
                .push_notification(message, NotificationType::Error, 16);
        }
        AppEvent::NetworkError(err) => {
            let message = format!("Network error: {}", err);
            app.state
                .ui
                .push_notification(message, NotificationType::Error, 16);
        }
        AppEvent::TapError(err) => {
            let message = format!("Tap error: {}", err);
            app.state
                .ui
                .push_notification(message, NotificationType::Error, 16);
        }
        AppEvent::UnknowCommand(err) => {
            let message = format!("Unknow command: {}", err);
            app.state.game.push_game_output(message.clone());
            app.state
                .ui
                .push_notification(message, NotificationType::Error, 16);
        }
        AppEvent::ClientConnected(client) => {
            app.state.net.client = Some(Arc::new(tokio::sync::Mutex::new(client)));
            app.state
                .game
                .push_game_output("TCP connection established.".to_string());
        }
        AppEvent::ApiDisconnected => {
            app.state.net.connection_state = ConnectionState::Disconnected;
            app.switch_to_login();
            if app.state.net.client.is_some() {
                app.state.ui.push_notification(
                    "Disconnected from server".to_string(),
                    NotificationType::Error,
                    16,
                );
                app.state.net.client = None;
            }
        }
        AppEvent::InventoryUpdate(items) => {
            app.state.game.inventory = items;
        }
        AppEvent::UpdateOnlinePlayers(count) => {
            app.state.game.online_players = count;
        }
        AppEvent::LocalChatSent(scope, msg) => {
            let sender =
                if let ConnectionState::Connected(name) = &app.state.net.connection_state {
                    format!("{} (You)", name)
                } else {
                    "You".to_string()
                };
            app.state.game.push_chat(scope, sender, msg);
        }
        AppEvent::UpdateGroup(group_name) => {
            app.state.game.group_name = group_name;
        }
        AppEvent::UpdateRoomContext {
            room_id,
            room_display_name,
            npcs,
        } => {
            app.state.game.current_room = room_id;
            app.state.game.current_room_name = room_display_name;
            app.state.game.npcs_in_room = npcs;
        }
        AppEvent::UpdateStatus { hp, max_hp } => {
            app.state.game.hp = hp;
            app.state.game.max_hp = max_hp;
        }
        AppEvent::Api(api_evt) => {
            network::handle_server_event(&mut app.state, api_evt);
        }
    }
}
