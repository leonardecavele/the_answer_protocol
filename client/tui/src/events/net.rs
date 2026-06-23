use crate::app::App;
use crate::events::{AppEvent, GameEvent, NetEvent, UiEvent};
use crate::network;
use crate::state::NotificationType;
use tokio::sync::mpsc;

pub enum NetworkCommand {
    ExecuteConnect(String),
    RequestWho,
}

pub async fn handle(event: NetEvent, app: &mut App, tx: &mpsc::Sender<AppEvent>) {
    match event {
        NetEvent::AttemptConnect(name, ip, port) => {
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

                        client.on_event(move |ev| {
                            let _ = tx_ev.send(AppEvent::Net(NetEvent::Api(ev)));
                        });

                        let (net_tx, mut net_rx) = mpsc::channel::<NetworkCommand>(100);

                        let _ = tx_clone.send(AppEvent::Net(NetEvent::ClientConnected(net_tx)));
                        let _ = tx_clone.send(AppEvent::Net(NetEvent::ExecuteConnect(name)));

                        while let Some(cmd) = net_rx.recv().await {
                            match cmd {
                                NetworkCommand::ExecuteConnect(player_name) => {
                                    match client.connect(player_name.clone()).await {
                                        Ok(Ok(_)) => {
                                            let _ = tx_clone.send(AppEvent::Net(
                                                NetEvent::LoginSuccess(player_name),
                                            ));
                                        }
                                        Ok(Err(err)) => {
                                            let _ = tx_clone.send(AppEvent::Game(
                                                GameEvent::CommandError(err),
                                            ));
                                            let _ = tx_clone.send(AppEvent::Net(
                                                NetEvent::ApiDisconnected,
                                            ));
                                        }
                                        Err(err) => {
                                            let _ = tx_clone.send(AppEvent::Net(
                                                NetEvent::TapError(err),
                                            ));
                                            let _ = tx_clone.send(AppEvent::Net(
                                                NetEvent::ApiDisconnected,
                                            ));
                                        }
                                    }
                                }
                                NetworkCommand::RequestWho => {
                                    if let Ok(Ok(data)) = client.who().await {
                                        let _ = tx_clone.send(AppEvent::Game(
                                            GameEvent::UpdateOnlinePlayers(data.player_count),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx_clone.send(AppEvent::Net(NetEvent::TapError(e)));
                        let _ = tx_clone.send(AppEvent::Net(NetEvent::ApiDisconnected));
                    }
                }
            });
        }
        NetEvent::ExecuteConnect(name) => {
            if let Some(net_tx) = &app.state.net.network_tx {
                let _ = net_tx.send(NetworkCommand::ExecuteConnect(name)).await;
            }
        }
        NetEvent::LoginSuccess(name) => {
            app.switch_to_game();
            let _ = tx.send(AppEvent::Game(GameEvent::PushGameOutput(format!(
                "Welcome, {}!",
                name
            ))));

            if let Some(net_tx) = &app.state.net.network_tx {
                let _ = net_tx.send(NetworkCommand::RequestWho).await;
            }
        }
        NetEvent::NetworkError(err) => {
            let _ = tx.send(AppEvent::Ui(UiEvent::Notification(
                format!("Network error: {}", err),
                NotificationType::Error,
                16,
            )));
        }
        NetEvent::TapError(err) => {
            let _ = tx.send(AppEvent::Ui(UiEvent::Notification(
                format!("Tap error: {}", err),
                NotificationType::Error,
                16,
            )));
        }
        NetEvent::ClientConnected(net_tx) => {
            app.state.net.network_tx = Some(net_tx);
            let _ = tx.send(AppEvent::Game(GameEvent::PushGameOutput(
                "TCP connection established.".to_string(),
            )));
        }
        NetEvent::ApiDisconnected => {
            app.state.net.network_tx = None;
            app.switch_to_login();
            let _ = tx.send(AppEvent::Ui(UiEvent::Notification(
                "Disconnected from server".to_string(),
                NotificationType::Error,
                16,
            )));
        }
        NetEvent::Api(api_evt) => {
            network::handle_server_event(&mut app.state, api_evt);
        }
    }
}