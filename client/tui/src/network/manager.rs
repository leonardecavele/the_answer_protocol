use crate::events::{ApplicationEvent, NetworkEvent};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// The NetworkManager is responsible for handling the connection to the game server.
/// It runs in a background Tokio task to ensure the UI never freezes during I/O.
pub struct NetworkManager {
    /// Handle to the background networking task.
    background_task: JoinHandle<()>,
}

impl NetworkManager {
    /// Spawns the background network task.
    /// Takes a clone of the event broker sender to push network events to the main loop.
    pub fn start(
        event_sender: mpsc::Sender<ApplicationEvent>,
        server_ip: String,
        server_port: String,
        player_name: String,
    ) -> Self {
        let background_task = tokio::spawn(async move {
            let server_address = format!("{}:{}", server_ip, server_port);

            match api_client::client::connect::ClientConnect::connect(&server_address).await {
                Ok(mut client) => {
                    let _ = event_sender.send(ApplicationEvent::Network(NetworkEvent::ConnectionEstablished {
                        server_ip,
                        server_port,
                        player_name,
                    })).await;

                    client.on_event({
                        let _event_sender = event_sender.clone();
                        move |server_event| {
                            tracing::debug!("Received event from server: {:?}", server_event);
                        }
                    });

                    let (_tx, rx) = tokio::sync::oneshot::channel::<()>();
                    let _ = rx.await;
                }
                Err(e) => {
                    let _ = event_sender.send(ApplicationEvent::Network(NetworkEvent::ConnectionFailed {
                        error_message: e.to_string(),
                    })).await;
                }
            }
        });

        Self { background_task }
    }
}

impl Drop for NetworkManager {
    fn drop(&mut self) {
        self.background_task.abort();
    }
}
