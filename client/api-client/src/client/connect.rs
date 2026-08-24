use crate::client::BridgeHandle;
use crate::client::bridge::{Bridge, BridgeChannels};
use crate::events::ServerEvent;
use crate::protocol::handshake::HandshakeResponse;
use crate::protocol::request::Request;
use crate::{
    Client, ClientConfig, Connection, ConnectionState, Frame, FrameDirection, NetworkError,
    ProtocolError, ServerInfo, ServerResponse, TapError,
};
use futures::StreamExt;
use std::fmt::Display;
use std::time::Duration;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::{broadcast, mpsc, watch};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::codec::{Framed, LinesCodec};
use tokio_util::sync::CancellationToken;
use tracing::info;

const SUPPORTED_PROTOCOL: u32 = 1;

impl Client {
    pub async fn connect<A>(addr: A) -> Result<Connection, TapError>
    where
        A: ToSocketAddrs + Clone + Display,
    {
        Self::connect_with(addr, ClientConfig::default()).await
    }

    pub async fn connect_with<A>(addr: A, config: ClientConfig) -> Result<Connection, TapError>
    where
        A: ToSocketAddrs + Clone + Display,
    {
        let (mut socket, server_addr) =
            Self::connect_tcp(addr, config.connect_timeout, config.max_frame_length).await?;
        info!("successfully connected to TCP socket at {}", server_addr);

        let (frame_sender, frame_receiver) =
            broadcast::channel::<Frame>(config.frame_channel_capacity);

        let handshake =
            Self::handshake(&mut socket, frame_sender.clone(), config.handshake_timeout).await?;
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

        let (state_sender, state_receiver) = watch::channel(ConnectionState::Connected);
        let (request_sender, request_receiver) =
            mpsc::channel::<Request>(config.command_channel_capacity);
        let (event_sender, event_receiver) =
            broadcast::channel::<ServerEvent>(config.event_channel_capacity);
        let cancellation = CancellationToken::new();

        let channels = BridgeChannels {
            state: state_sender.clone(),
            events: event_sender.clone(),
            frames: frame_sender.clone(),
            requests: request_receiver,
        };

        let bridge_task = Self::start_bridge(
            socket,
            channels,
            cancellation.clone(),
            config.request_timeout,
        );

        let client = Client {
            server: ServerInfo {
                addr: server_addr,
                protocol_version: handshake.server_protocol_version,
            },
            bridge: BridgeHandle {
                task: bridge_task,
                request_sender,
                frame_sender,
                event_sender,
                cancellation,
            },
            close_timeout: config.close_timeout,
            state: state_receiver,
        };

        Ok(Connection {
            client,
            events: event_receiver,
            frames: frame_receiver,
        })
    }

    async fn connect_tcp<A>(
        addr: A,
        connect_timeout: Duration,
        max_frame_length: usize,
    ) -> Result<(Framed<TcpStream, LinesCodec>, String), NetworkError>
    where
        A: ToSocketAddrs + Clone + Display,
    {
        info!("try connection to server {}...", addr);

        match timeout(connect_timeout, TcpStream::connect(addr.clone())).await {
            Ok(Ok(stream)) => {
                stream.set_nodelay(true)?;
                let peer_addr = stream.peer_addr()?.to_string();
                let socket = Framed::new(stream, LinesCodec::new_with_max_length(max_frame_length));
                Ok((socket, peer_addr))
            }
            Ok(Err(io_error)) => Err(NetworkError::ConnectionFailed {
                addr: addr.to_string(),
                source: io_error,
            }),
            Err(_) => Err(NetworkError::ConnectionTimeout {
                addr: addr.to_string(),
                timeout: connect_timeout,
            }),
        }
    }

    fn start_bridge(
        socket: Framed<TcpStream, LinesCodec>,
        channels: BridgeChannels,
        cancellation: CancellationToken,
        request_timeout: Duration,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut bridge = Bridge::new(socket, channels, request_timeout);
            bridge.listen(cancellation).await;
        })
    }

    async fn handshake(
        socket: &mut Framed<TcpStream, LinesCodec>,
        frame_sender: broadcast::Sender<Frame>,
        handshake_timeout: Duration,
    ) -> Result<HandshakeResponse, TapError> {
        let frame = timeout(handshake_timeout, socket.next())
            .await
            .map_err(|_| {
                TapError::Network(NetworkError::HandshakeTimeout {
                    timeout: handshake_timeout,
                })
            })?;

        match frame {
            Some(Ok(line)) => {
                let _ = frame_sender.send(Frame {
                    direction: FrameDirection::Received,
                    line: line.clone(),
                });
                let response = ServerResponse::try_from(line)?;
                Ok(HandshakeResponse::try_from(response)?)
            }
            Some(Err(codec_error)) => Err(NetworkError::Codec(codec_error).into()),
            None => Err(NetworkError::Disconnected.into()),
        }
    }
}
