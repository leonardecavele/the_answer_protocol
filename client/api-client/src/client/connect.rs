use crate::client::bridge::Bridge;
use crate::client::event::ServerEvent;
use crate::client::{BridgeHandle, Client, ServerInfo};
use crate::error::{InternalError, NetworkError, ProtocolError, TapError};
use crate::protocol::handshake::HandshakeResponse;
use crate::protocol::request::Request;
use crate::protocol::response::ServerResponse;
use futures::StreamExt;
use std::fmt::Display;
use std::time::Duration;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::codec::{Framed, LinesCodec};
use tokio_util::sync::CancellationToken;
use tracing::info;

const SUPPORTED_PROTOCOL: u32 = 1;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(2);

pub struct ClientConnect;

impl ClientConnect {
    pub async fn connect<A>(addr: A) -> Result<(Client, broadcast::Receiver<ServerEvent>), TapError>
    where
        A: ToSocketAddrs + Clone + Display,
    {
        let (mut socket, server_addr) = Self::connect_tcp(addr).await?;
        info!("successfully connected to TCP socket at {}", server_addr);

        let (request_sender, request_receiver) = mpsc::channel::<Request>(2048);
        let (event_sender, event_receiver) = broadcast::channel::<ServerEvent>(2048);
        let cancellation = CancellationToken::new();

        let handshake = Self::handshake(&mut socket, HANDSHAKE_TIMEOUT).await?;
        info!(
            "handshake successful, protocol version: {}",
            handshake.server_protocol_version
        );

        if handshake.server_protocol_version != SUPPORTED_PROTOCOL {
            return Err(ProtocolError::UnsupportedVersion {
                server: handshake.server_protocol_version,
                supported: SUPPORTED_PROTOCOL,
            }
            .into());
        }

        let bridge_handler = Self::start_bridge(
            socket,
            event_sender.clone(),
            request_receiver,
            cancellation.clone(),
        )
        .await?;

        let client = Client {
            server: ServerInfo {
                addr: server_addr,
                protocol_version: handshake.server_protocol_version,
            },
            bridge: BridgeHandle {
                task: bridge_handler,
                command_sender: request_sender,
                event_sender,
                cancellation,
            },
        };

        Ok((client, event_receiver))
    }

    async fn connect_tcp<A>(
        addr: A,
    ) -> Result<(Framed<TcpStream, LinesCodec>, String), NetworkError>
    where
        A: ToSocketAddrs + Clone + Display,
    {
        info!("try connection to server {}...", addr);

        // let max_attempt: u32 = u32::MAX;
        let max_attempt: u32 = 3;
        let timeout_before_retry_ms: u64 = 1000;

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
                        addr, timeout_before_retry_ms
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
                        addr, timeout_before_retry_ms
                    );
                }
            }

            tokio::time::sleep(Duration::from_millis(timeout_before_retry_ms)).await;
        }

        Err(NetworkError::ConnectionTimeout {
            addr: addr.to_string(),
        })
    }

    async fn start_bridge(
        socket: Framed<TcpStream, LinesCodec>,
        event_sender: broadcast::Sender<ServerEvent>,
        command_receiver: mpsc::Receiver<Request>,
        cancellation: CancellationToken,
    ) -> Result<JoinHandle<()>, InternalError> {
        let bridge_task = tokio::spawn(async move {
            let mut bridge = Bridge::new(socket, event_sender, command_receiver);
            bridge.listen(cancellation).await;
        });

        Ok(bridge_task)
    }

    async fn handshake(
        socket: &mut Framed<TcpStream, LinesCodec>,
        timeout_duration: Duration,
    ) -> Result<HandshakeResponse, TapError> {
        let frame = timeout(timeout_duration, socket.next())
            .await
            .map_err(|_| {
                TapError::Network(NetworkError::HandshakeTimeout {
                    timeout: HANDSHAKE_TIMEOUT,
                })
            })?;

        match frame {
            Some(Ok(line)) => Ok(HandshakeResponse::try_from(ServerResponse::try_from(
                line,
            )?)?),
            Some(Err(codec_error)) => Err(NetworkError::Codec(codec_error).into()),
            None => Err(NetworkError::Disconnected.into()),
        }
    }
}
