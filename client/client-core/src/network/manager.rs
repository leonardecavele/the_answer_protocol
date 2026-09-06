use super::request::RequestChain;
use crate::events::{ApiEvent, ApplicationEvent, ConnectionEvent};
use client_api::events::ServerEvent;
use client_api::{Client, Connection, ConnectionState, Frame};
use mpsc::Sender;
use tokio::sync::broadcast::error::{RecvError, TryRecvError};
use tokio::sync::{broadcast, mpsc, watch};
use tokio_util::task::AbortOnDropHandle;
use tracing::info;

pub struct NetworkManager {
    command_sender: Sender<RequestChain>,
    _background_task: AbortOnDropHandle<()>,
}

impl NetworkManager {
    pub fn start(
        event_sender: Sender<ApplicationEvent>,
        server_ip: String,
        server_port: String,
        player_name: String,
    ) -> Self {
        let (command_tx, command_rx) = mpsc::channel::<RequestChain>(128);

        let _background_task = AbortOnDropHandle::new(tokio::spawn(async move {
            let result = Self::run(
                &event_sender,
                command_rx,
                server_ip,
                server_port,
                player_name,
            )
            .await;

            if let Err(error_message) = result {
                let _ = event_sender
                    .send(ApplicationEvent::Connection(ConnectionEvent::Failed {
                        error_message,
                    }))
                    .await;
            }
        }));

        Self {
            command_sender: command_tx,
            _background_task,
        }
    }

    pub fn send_command(&self, chain: RequestChain) -> Result<(), RequestChain> {
        self.command_sender
            .try_send(chain)
            .map_err(|error| error.into_inner())
    }

    async fn run(
        event_sender: &Sender<ApplicationEvent>,
        mut command_rx: mpsc::Receiver<RequestChain>,
        server_ip: String,
        server_port: String,
        player_name: String,
    ) -> Result<(), String> {
        let address = format!("{}:{}", server_ip, server_port);

        let Connection {
            mut client,
            events,
            mut frames,
        } = Client::connect(&address).await.map_err(|e| e.to_string())?;

        let login_result = client.login(player_name.clone()).await;

        Self::drain_frames(event_sender, &mut frames).await;

        login_result
            .map_err(|error| error.to_string())?
            .map_err(|error| error.message)?;

        let _ = event_sender
            .send(ApplicationEvent::Connection(ConnectionEvent::Established {
                server_ip,
                server_port,
                player_name,
            }))
            .await;

        let _forward_task = AbortOnDropHandle::new(tokio::spawn(Self::forward_events(
            event_sender.clone(),
            frames,
            events,
            client.state(),
        )));

        Self::run_commands(event_sender, &mut command_rx, &client).await;

        Ok(())
    }

    async fn send_lagged(
        event_sender: &Sender<ApplicationEvent>,
        stream: &'static str,
        count: u64,
    ) {
        let _ = event_sender
            .send(ApplicationEvent::Api(ApiEvent::Lagged {
                stream,
                count: count as usize,
            }))
            .await;
    }

    async fn drain_frames(
        event_sender: &Sender<ApplicationEvent>,
        frames: &mut broadcast::Receiver<Frame>,
    ) {
        loop {
            match frames.try_recv() {
                Ok(frame) => {
                    let _ = event_sender
                        .send(ApplicationEvent::Api(ApiEvent::Frame(frame)))
                        .await;
                }
                Err(TryRecvError::Lagged(count)) => {
                    Self::send_lagged(event_sender, "frame", count).await;
                }
                Err(_) => break,
            }
        }
    }

    async fn forward_events(
        event_sender: Sender<ApplicationEvent>,
        mut frames: broadcast::Receiver<Frame>,
        mut events: broadcast::Receiver<ServerEvent>,
        mut client_state: watch::Receiver<ConnectionState>,
    ) {
        loop {
            tokio::select! {
                frame = frames.recv() => match frame {
                    Ok(frame) => {
                        let _ = event_sender
                            .send(ApplicationEvent::Api(ApiEvent::Frame(frame)))
                            .await;
                    }
                    Err(RecvError::Lagged(count)) => {
                        Self::send_lagged(&event_sender, "frame", count).await;
                    }
                    Err(RecvError::Closed) => {
                        info!("connection closed");
                        break;
                    }
                },
                event = events.recv() => match event {
                    Ok(server_event) => {
                        let _ = event_sender
                            .send(ApplicationEvent::Api(ApiEvent::Server(server_event)))
                            .await;
                    }
                    Err(RecvError::Lagged(count)) => {
                        Self::send_lagged(&event_sender, "event", count).await;
                    }
                    Err(RecvError::Closed) => {
                        info!("connection closed");
                        break;
                    }
                },
                _ = client_state.changed() => {
                    info!("connection closed");

                    let state = client_state.borrow().clone();

                    if let ConnectionState::Lost(reason) = state {
                        let _ = event_sender
                            .send(ApplicationEvent::Connection(ConnectionEvent::Lost { reason }))
                            .await;
                    }

                    break;
                },
            }
        }
    }

    async fn run_commands(
        event_sender: &Sender<ApplicationEvent>,
        command_rx: &mut mpsc::Receiver<RequestChain>,
        client: &Client,
    ) {
        while let Some(chain) = command_rx.recv().await {
            for request in chain {
                let original_request = request.clone();

                match client.execute_request(request).await {
                    Ok(response) => {
                        let is_failed = response.get_error().is_some();

                        let _ = event_sender
                            .send(ApplicationEvent::Api(ApiEvent::ApiResponse {
                                response,
                                original_request,
                            }))
                            .await;

                        if is_failed {
                            break;
                        }
                    }
                    Err(tap_error) => {
                        let _ = event_sender
                            .send(ApplicationEvent::Api(ApiEvent::RequestFailed {
                                request: original_request,
                                error_message: tap_error.to_string(),
                            }))
                            .await;

                        break;
                    }
                }
            }
        }
    }
}
