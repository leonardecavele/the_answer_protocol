use crate::client::dispatcher::EventDispatcher;
use crate::client::{BridgeState, Client, ServerInfo};
use crate::error::{InternalError, NetworkError, TapError};
use crate::network::bridge::Bridge;
use crate::protocol::handshake::HandshakeResponse;
use crate::protocol::request::Request;
use crate::protocol::response::ServerResponse;
use std::fmt::Display;
use std::time::Duration;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{debug, info};

pub struct ClientConnect;

impl ClientConnect {
    pub async fn connect<A>(addr: A) -> Result<Client, TapError>
    where
        A: ToSocketAddrs + Clone + Display,
    {
        let (socket, server_addr) = Self::connect_tcp(addr).await?;
        info!("successfully connected to TCP socket at {}", server_addr);

        let (request_sender, request_receiver) = mpsc::channel::<Request>(2048);
        let (event_broadcast_sender, _) = broadcast::channel::<ServerResponse>(2048);
        let (handshake_request, handshake_receiver) = Request::handshake();
        let bridge_handler = Self::start_bridge(
            socket,
            handshake_request,
            event_broadcast_sender.clone(),
            request_receiver,
        )
        .await?;

        debug!("awaiting server handshake...");
        let handshake = Self::await_handshake(handshake_receiver).await?;
        info!(
            "handshake successful, protocol version: {}",
            handshake.server_protocol_version
        );

        Ok(Client {
            server: ServerInfo {
                addr: server_addr,
                protocol_version: handshake.server_protocol_version,
            },
            bridge: BridgeState {
                bridge_task: bridge_handler,
                command_sender: request_sender,
            },
            event_dispatcher: EventDispatcher::new(event_broadcast_sender),
        })
    }

    async fn connect_tcp<A: ToSocketAddrs + Clone + Display>(
        addr: A,
    ) -> Result<(Framed<TcpStream, LinesCodec>, String), NetworkError>
    where
        A: ToSocketAddrs + Clone + Display,
    {
        info!("try connection to server {}...", addr);

        // let max_attempt: u32 = u32::MAX;
        let max_attempt: u32 = 3;
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
                        return Err(NetworkError::ConnectionMaxRetry {
                            addr: addr.to_string(),
                        });
                    }
                    info!(
                        "failed to connect to {}, retrying in {} milliseconds..",
                        addr, timeout_before_retry
                    );
                }
                Err(_) => {
                    if attempt >= max_attempt {
                        return Err(NetworkError::ConnectionMaxRetry {
                            addr: addr.to_string(),
                        });
                    }
                    info!(
                        "connection to {} timed out, retrying in {} milliseconds..",
                        addr, timeout_before_retry
                    );
                }
            }

            tokio::time::sleep(Duration::from_millis(timeout_before_retry)).await;
        }

        Err(NetworkError::ConnectionTimeout {
            addr: addr.to_string(),
        })
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
}
