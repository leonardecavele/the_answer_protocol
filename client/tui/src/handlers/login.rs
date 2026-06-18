use crate::app::{App, ConnectionState, LoginField};
use crate::events::AppEvent;
use crossterm::event::{Event, KeyCode, KeyEvent};
use tokio::sync::mpsc;

pub fn handle_key(
    app: &mut App,
    key: KeyEvent,
    evt: &Event,
    tx: &mpsc::UnboundedSender<AppEvent>,
) {
    match key.code {
        KeyCode::Tab => {
            app.login_field = match app.login_field {
                LoginField::Username => LoginField::Address,
                LoginField::Address => LoginField::Port,
                LoginField::Port => {
                    if matches!(app.state, ConnectionState::Disconnected) {
                        LoginField::Button
                    } else {
                        LoginField::Username
                    }
                }
                LoginField::Button => LoginField::Username,
            };
        }
        KeyCode::BackTab => {
            app.login_field = match app.login_field {
                LoginField::Username => {
                    if matches!(app.state, ConnectionState::Disconnected) {
                        LoginField::Button
                    } else {
                        LoginField::Port
                    }
                }
                LoginField::Address => LoginField::Username,
                LoginField::Port => LoginField::Address,
                LoginField::Button => LoginField::Port,
            };
        }
        KeyCode::Enter => {
            if matches!(app.login_field, LoginField::Button)
                && matches!(app.state, ConnectionState::Disconnected)
            {
                let username = app.username_input.value().to_string();
                let ip = app.address_input.value().to_string();
                let port = app.port_input.value().to_string();

                if username.is_empty() || ip.is_empty() || port.is_empty() {
                    return;
                }

                app.server_ip = ip.clone();
                app.server_port = port.clone();
                app.state = ConnectionState::Connecting;

                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    use api_client::client::connect::ClientConnect;
                    match ClientConnect::connect(format!("{}:{}", ip, port)).await {
                        Ok(mut client) => {
                            let tx_ev = tx_clone.clone();
                            client.on_event(move |ev| {
                                let _ = tx_ev.send(AppEvent::Api(ev));
                            });
                            let _ = tx_clone.send(AppEvent::ClientConnected(client));
                            let _ = tx_clone.send(AppEvent::ExecuteConnect(username));
                        }
                        Err(e) => {
                            log::error!("TCP Connection failed: {}", e);
                            let _ = tx_clone.send(AppEvent::ApiDisconnected);
                        }
                    }
                });
            }
        }
        _ => match app.login_field {
            LoginField::Username => {
                tui_input::backend::crossterm::EventHandler::handle_event(
                    &mut app.username_input,
                    evt,
                );
            }
            LoginField::Address => {
                tui_input::backend::crossterm::EventHandler::handle_event(
                    &mut app.address_input,
                    evt,
                );
            }
            LoginField::Port => {
                tui_input::backend::crossterm::EventHandler::handle_event(&mut app.port_input, evt);
            }
            LoginField::Button => {}
        },
    }
}
