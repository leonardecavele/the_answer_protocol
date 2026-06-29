use crate::events::{ApiEvent, ApplicationEvent, NetworkEvent};
use crate::network::envelopes::{RequestEnvelope, ResponseEnvelope};
use api_client::client::connect::ClientConnect;
use mpsc::Sender;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub const NOTIF_ID_CONNECTION_ATTEMPT: &str = "notif_connection_attempt";

pub struct NetworkManager {
    background_task: JoinHandle<()>,
    pub command_sender: Sender<RequestEnvelope>,
}

impl NetworkManager {
    pub fn start(
        event_sender: Sender<ApplicationEvent>,
        server_ip: String,
        server_port: String,
        player_name: String,
    ) -> Self {
        let (command_tx, mut command_rx) = mpsc::channel::<RequestEnvelope>(128);

        let background_task = tokio::spawn(async move {
            let server_address = format!("{}:{}", server_ip, server_port);

            match ClientConnect::connect(&server_address).await {
                Ok(mut client) => {
                    match client.connect(player_name.clone()).await {
                        Ok(Ok(_connect_response)) => {
                            let _ = event_sender
                                .send(ApplicationEvent::Network(
                                    NetworkEvent::ConnectionEstablished {
                                        server_ip,
                                        server_port,
                                        player_name,
                                    },
                                ))
                                .await;

                            client.on_event({
                                let _event_sender = event_sender.clone();
                                move |server_event| {
                                    tracing::debug!(
                                        "Received event from server: {:?}",
                                        server_event
                                    );
                                    let _ = _event_sender.try_send(ApplicationEvent::Api(
                                        ApiEvent::Server(server_event),
                                    ));
                                }
                            });

                            // Command loop
                            while let Some(envelope) = command_rx.recv().await {
                                let original_request = envelope.request.clone();

                                let _ = event_sender.try_send(ApplicationEvent::Api(
                                    ApiEvent::LogApiRequest(envelope.clone()),
                                ));

                                match client.execute_request(envelope.request).await {
                                    Ok(api_response) => {
                                        let _ = event_sender.try_send(ApplicationEvent::Api(
                                            ApiEvent::ApiResponse(ResponseEnvelope {
                                                id: envelope.id,
                                                response: api_response,
                                                original_request,
                                            }),
                                        ));
                                    }
                                    Err(tap_error) => {
                                        let _ = event_sender.try_send(ApplicationEvent::Network(
                                            NetworkEvent::ConnectionLost {
                                                reason: format!(
                                                    "Failed to send request: {:?}",
                                                    tap_error
                                                ),
                                            },
                                        ));
                                        break;
                                    }
                                }
                            }
                        }
                        Ok(Err(command_error)) => {
                            let _ = event_sender
                                .send(ApplicationEvent::Network(NetworkEvent::ConnectionFailed {
                                    error_message: command_error.message,
                                }))
                                .await;
                        }
                        Err(tap_error) => {
                            let _ = event_sender
                                .send(ApplicationEvent::Network(NetworkEvent::ConnectionFailed {
                                    error_message: format!("Communication error: {:?}", tap_error),
                                }))
                                .await;
                        }
                    }
                }
                Err(e) => {
                    let _ = event_sender
                        .send(ApplicationEvent::Network(NetworkEvent::ConnectionFailed {
                            error_message: format!("TCP error: {}", e),
                        }))
                        .await;
                }
            }
        });

        Self {
            background_task,
            command_sender: command_tx,
        }
    }

    pub fn send_command(&self, cmd: RequestEnvelope) {
        let _ = self.command_sender.try_send(cmd);
    }
}

impl Drop for NetworkManager {
    fn drop(&mut self) {
        self.background_task.abort();
    }
}
