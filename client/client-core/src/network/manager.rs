use super::envelopes::{RequestEnvelope, ResponseEnvelope};
use crate::events::{ApiEvent, ApplicationEvent, NetworkConnectionEvent};
use client_api::{Client, Connection, ConnectionState};
use mpsc::Sender;
use tokio::sync::broadcast::error::{RecvError, TryRecvError};
use tokio::sync::mpsc;
use tokio_util::task::AbortOnDropHandle;
use tracing::{info, warn};

pub struct NetworkManager {
    pub command_sender: Sender<RequestEnvelope>,
    _background_task: AbortOnDropHandle<()>,
}

impl NetworkManager {
    pub fn start(
        event_sender: Sender<ApplicationEvent>,
        server_ip: String,
        server_port: String,
        player_name: String,
    ) -> Self {
        let (command_tx, mut command_rx) = mpsc::channel::<RequestEnvelope>(128);

        let _background_task = AbortOnDropHandle::new(tokio::spawn(async move {
            let server_address = format!("{}:{}", server_ip, server_port);

            match Client::connect(&server_address).await {
                Ok(Connection {
                    mut client,
                    mut events,
                    mut frames,
                }) => {
                    let login_result = client.login(player_name.clone()).await;

                    loop {
                        match frames.try_recv() {
                            Ok(frame) => {
                                let _ = event_sender
                                    .send(ApplicationEvent::Api(ApiEvent::Frame(frame)))
                                    .await;
                            }
                            Err(TryRecvError::Lagged(count)) => {
                                let _ = event_sender
                                    .send(ApplicationEvent::Api(ApiEvent::Lagged {
                                        stream: "frame",
                                        count: count as usize,
                                    }))
                                    .await;
                            }
                            Err(_) => break,
                        }
                    }

                    match login_result {
                        Ok(Ok(_)) => {
                            let _ = event_sender
                                .send(ApplicationEvent::Network(
                                    NetworkConnectionEvent::Established {
                                        server_ip,
                                        server_port,
                                        player_name,
                                    },
                                ))
                                .await;

                            let sender = event_sender.clone();
                            let mut client_state = client.state();
                            let _event_forward_task = AbortOnDropHandle::new(tokio::spawn(
                                async move {
                                    loop {
                                        tokio::select! {
                                            frame_recv = frames.recv() => {
                                                match frame_recv {
                                                    Ok(frame) => {
                                                        let _ = sender
                                                            .send(ApplicationEvent::Api(ApiEvent::Frame(frame)))
                                                            .await;
                                                    }
                                                    Err(RecvError::Lagged(count)) => {
                                                        let _ = sender
                                                            .send(ApplicationEvent::Api(ApiEvent::Lagged {
                                                            stream: "frame",
                                                            count: count as usize
                                                        }))
                                                            .await;
                                                    }
                                                    Err(RecvError::Closed) => {
                                                        info!("connection closed");
                                                        break;
                                                    }
                                                }
                                            },
                                            event = events.recv() => {
                                                match event {
                                                    Ok(server_event) => {
                                                        let _ = sender
                                                            .send(ApplicationEvent::Api(ApiEvent::Server(
                                                                server_event,
                                                            )))
                                                            .await;
                                                    }
                                                    Err(RecvError::Lagged(count)) => {
                                                        let _ = sender
                                                            .send(ApplicationEvent::Api(ApiEvent::Lagged {
                                                            stream: "event",
                                                            count: count as usize
                                                        }))
                                                            .await;
                                                    }
                                                    Err(RecvError::Closed) => {
                                                        info!("connection closed");
                                                        break;
                                                    }
                                                }
                                            },
                                            _state = client_state.changed() => {
                                                info!("connection closed");

                                                let state = client_state.borrow().clone();
                                                match state {
                                                    ConnectionState::Lost(reason) => {
                                                        let _ = sender
                                                            .send(ApplicationEvent::Network(NetworkConnectionEvent::Lost {
                                                                reason,
                                                            })).await;
                                                    },
                                                    ConnectionState::Connected => {},
                                                    ConnectionState::Closed => {}
                                                }

                                                break;
                                            },
                                        }
                                    }
                                },
                            ));

                            while let Some(envelope) = command_rx.recv().await {
                                let original_request = envelope.request.clone();

                                match client.execute_request(envelope.request).await {
                                    Ok(api_response) => {
                                        let _ = event_sender
                                            .send(ApplicationEvent::Api(ApiEvent::ApiResponse(
                                                ResponseEnvelope {
                                                    id: envelope.id,
                                                    response: api_response,
                                                    original_request,
                                                },
                                            )))
                                            .await;
                                    }
                                    Err(tap_error) => {
                                        warn!("command failed: {}", tap_error);
                                    }
                                }
                            }
                        }
                        Ok(Err(command_error)) => {
                            let _ = event_sender
                                .send(ApplicationEvent::Network(NetworkConnectionEvent::Failed {
                                    error_message: command_error.message,
                                }))
                                .await;
                        }
                        Err(tap_error) => {
                            let _ = event_sender
                                .send(ApplicationEvent::Network(NetworkConnectionEvent::Failed {
                                    error_message: tap_error.to_string(),
                                }))
                                .await;
                        }
                    }
                }
                Err(e) => {
                    let _ = event_sender
                        .send(ApplicationEvent::Network(NetworkConnectionEvent::Failed {
                            error_message: e.to_string(),
                        }))
                        .await;
                }
            }
        }));

        Self {
            command_sender: command_tx,
            _background_task,
        }
    }

    pub fn send_command(&self, cmd: RequestEnvelope) {
        let _ = self.command_sender.try_send(cmd);
    }
}
