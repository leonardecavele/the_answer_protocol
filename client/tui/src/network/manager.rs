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
    ) -> Self {
        let background_task = tokio::spawn(async move {
            let server_address = format!("{}:{}", server_ip, server_port);
            
            // Notify the application that we are starting the connection attempt.
            let _ = event_sender
                .send(ApplicationEvent::Network(NetworkEvent::ConnectionAttemptStarted {
                    server_address: server_address.clone(),
                }))
                .await;

            // TODO: In the future, this is where we will instantiate `api_client::client::connect::ClientConnect`
            // and actually perform the TCP connection.
            
            // For now, let's simulate a connection delay to prove the async nature.
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            // Simulate a connection failure just for the scaffolding phase.
            // Normally we would loop here listening to `api_client` events and translating them.
            let _ = event_sender
                .send(ApplicationEvent::Network(NetworkEvent::ConnectionFailed {
                    error_message: "Network layer not fully implemented yet".to_string(),
                }))
                .await;
        });

        Self { background_task }
    }
}

impl Drop for NetworkManager {
    fn drop(&mut self) {
        // Ensure the network task is aborted if the manager is dropped
        self.background_task.abort();
    }
}
