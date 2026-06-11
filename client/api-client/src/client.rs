use crate::error::{CommandError, InternalError, NetworkError, TapError};
use crate::network::bridge::Bridge;
use crate::protocol::command::Command;
use crate::protocol::command::connect::{ConnectCommand, ConnectResponse};
use crate::protocol::command::look::{LookCommand, LookResponse};
use crate::protocol::command::quit::QuitCommand;
use crate::protocol::handshake::HandshakeResponse;
use crate::protocol::request::Request;
use crate::protocol::response::{Opcode, ServerResponse};
use std::fmt::Display;
use std::process::abort;
use std::time::Duration;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{Event, debug, info, warn};

#[derive(Debug)]
pub struct ServerInfo {
    pub addr: String,
    pub protocol_version: u32,
}

pub struct APIClient {
    pub server: ServerInfo,
    bridge: BridgeState,
}

struct BridgeState {
    bridge_task: JoinHandle<()>,
    command_sender: mpsc::Sender<Request>,
    event_dispatcher: EventDispatcher,
}

struct EventDispatcher {
    broadcast_sender: broadcast::Sender<ServerResponse>,
    subscriber_tasks: Vec<JoinHandle<()>>,
}

impl APIClient {
    pub async fn new<A>(addr: A) -> Result<APIClient, TapError>
    where
        A: ToSocketAddrs + Clone + Display,
    {
        let (socket, server_addr) = Self::connect_tcp(addr).await?;
        info!("successfully connected to TCP socket at {}", server_addr);

        let (request_sender, request_receiver) = mpsc::channel::<Request>(65536);
        let (event_sender, _) = broadcast::channel::<ServerResponse>(65536);

        let (handshake_request, handshake_receiver) = Request::handshake();
        let bridge_handler = Self::start_bridge(
            socket,
            handshake_request,
            event_sender.clone(),
            request_receiver,
        )
        .await?;

        debug!("awaiting server handshake...");
        let handshake = Self::await_handshake(handshake_receiver).await?;
        info!(
            "handshake successful, protocol version: {}",
            handshake.server_protocol_version
        );

        Ok(APIClient {
            server: ServerInfo {
                addr: server_addr,
                protocol_version: handshake.server_protocol_version,
            },
            bridge: BridgeState {
                bridge_task: bridge_handler,
                command_sender: request_sender,
                event_dispatcher: EventDispatcher {
                    broadcast_sender: event_sender,
                    subscriber_tasks: vec![],
                },
            },
        })
    }

    async fn connect_tcp<A: ToSocketAddrs + Clone + Display>(
        addr: A,
    ) -> Result<(Framed<TcpStream, LinesCodec>, String), NetworkError>
    where
        A: ToSocketAddrs + Clone + Display,
    {
        info!("try connection to server {}...", addr);

        let max_attempt: u32 = u32::MAX;
        let timeout_before_retry: u64 = 5;

        for attempt in 1..=max_attempt {
            let connection_future = TcpStream::connect(addr.clone());
            let timeout_duration = Duration::from_secs(5);

            match timeout(timeout_duration, connection_future).await {
                Ok(Ok(stream)) => {
                    stream.set_nodelay(true)?;
                    let peer_addr = stream.peer_addr()?.to_string();
                    let socket = Framed::new(stream, LinesCodec::new_with_max_length(65536));
                    return Ok((socket, peer_addr));
                }
                Ok(Err(_)) => {
                    if attempt >= max_attempt {
                        return Err(NetworkError::ConnectionMaxRetry);
                    }
                    info!(
                        "failed to connect to {}, retrying in {} milliseconds..",
                        addr, timeout_before_retry
                    );
                }
                Err(_) => {
                    if attempt >= max_attempt {
                        return Err(NetworkError::ConnectionTimeout);
                    }
                    info!(
                        "connection to {} timed out, retrying in {} milliseconds..",
                        addr, timeout_before_retry
                    );
                }
            }

            tokio::time::sleep(Duration::from_millis(timeout_before_retry)).await;
        }

        Err(NetworkError::Disconnected)
    }

    async fn start_bridge(
        socket: Framed<TcpStream, LinesCodec>,
        handshake_request: Request,
        event_sender: broadcast::Sender<ServerResponse>,
        command_receiver: mpsc::Receiver<Request>,
    ) -> Result<JoinHandle<()>, InternalError> {
        let (ready_sender, ready_receiver) = oneshot::channel::<()>();

        let bridge_task = tokio::spawn(async move {
            let mut bridge = Bridge::new(socket, event_sender, command_receiver);
            bridge.listen(handshake_request, ready_sender).await;
        });

        ready_receiver.await.map_err(|e| {
            InternalError::ThreadPanic(format!("bridge task panicked during initialization: {}", e))
        })?;

        Ok(bridge_task)
    }

    async fn await_handshake(
        handshake_receiver: oneshot::Receiver<ServerResponse>,
    ) -> Result<HandshakeResponse, TapError> {
        let response = handshake_receiver
            .await
            .map_err(|_| NetworkError::Disconnected)?;
        Ok(HandshakeResponse::try_from(response)?)
    }

    async fn request<C: Command>(
        &self,
        command: C,
    ) -> Result<Result<C::ResponseData, CommandError>, TapError> {
        match command.create_command(&self.server) {
            Ok(raw_command) => {
                let (request, response_receiver) = Request::new(raw_command);

                self.bridge
                    .command_sender
                    .send(request)
                    .await
                    .map_err(|e| {
                        InternalError::ChannelPanic(
                            "failed to send command to the bridge task (task may have crashed)"
                                .to_string(),
                        )
                    })?;

                let response = response_receiver.await.map_err(|e| {
                    InternalError::ChannelPanic(
                        "bridge task dropped the response channel without \
                        replying (connection probably died)"
                            .to_string(),
                    )
                })?;

                if response.opcode == Opcode::Ok {
                    Ok(command.parse_response(&self.server, response))
                } else {
                    Ok(Err(CommandError::from_response(response)))
                }
            }
            Err(e) => Ok(Err(e)),
        }
    }
}

impl Drop for APIClient {
    fn drop(&mut self) {
        self.bridge.bridge_task.abort();
        for event_subscriber in self.bridge.event_dispatcher.subscriber_tasks.iter() {
            event_subscriber.abort();
        }

        info!("APIClient dropped: background tasks aborted");
    }
}

impl APIClient {
    pub fn on_event<F>(&mut self, handler: F)
    where
        F: Fn(ServerResponse) + Send + 'static,
    {
        let mut subscriber = self.bridge.event_dispatcher.broadcast_sender.subscribe();

        self.bridge
            .event_dispatcher
            .subscriber_tasks
            .push(tokio::spawn(async move {
                loop {
                    match subscriber.recv().await {
                        Ok(event) => handler(event),
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("lag.. {} events dropped", skipped);
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            }))
    }

    pub async fn connect(
        &self,
        player_name: String,
    ) -> Result<Result<ConnectResponse, CommandError>, TapError> {
        debug!("sending connect request for player: {}", player_name);

        let response = self.request(ConnectCommand { player_name }).await?;

        Ok(response)
    }

    pub async fn look(&self) -> Result<Result<LookResponse, CommandError>, TapError> {
        debug!("sending look request");

        let response = self.request(LookCommand).await?;

        Ok(response)
    }

    pub async fn quit(self) {
        debug!("sending quit request");

        let _ = self.request(QuitCommand).await;
        drop(self)
    }
}
