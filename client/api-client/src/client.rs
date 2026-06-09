use crate::error::{TapError, TapResult};
use crate::network::bridge::Bridge;
use crate::protocol::command::{Command, connect};
use crate::protocol::envelope::Envelope;
use crate::protocol::packet::Packet;
use crate::protocol::packet::connect::ConnectPacket;
use crate::protocol::packet::handshake::HandshakePacket;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{debug, info};

#[derive(Debug)]
pub struct ServerInfo {
    pub addr: String,
    pub protocol_version: u32,
}

pub struct APIClient {
    pub server: ServerInfo,
    conn: Connection,
}

struct Connection {
    bridge_handle: JoinHandle<()>,
    sender: mpsc::Sender<Envelope>,
}

impl Connection {
    async fn send(&self, command: Command) -> TapResult<Packet> {
        let (tx, rx) = oneshot::channel::<Packet>();

        let envelope = Envelope {
            command,
            tx: Some(tx),
        };

        self.sender
            .send(envelope)
            .await
            .map_err(|e| TapError::Channel(format!("[client] send command error: {}", e)))?;

        rx.await
            .map_err(|e| TapError::Channel(format!("[client] recv command error: {}", e)))
    }
}

impl APIClient {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> TapResult<APIClient> {
        let (socket, server_addr) = Self::connect_tcp(addr).await?;
        info!("successfully connected to TCP socket at {}", server_addr);

        let (tx, rx) = mpsc::channel::<Envelope>(1024);
        let (handshake_tx, handshake_rx) = oneshot::channel::<Packet>();

        let (bridge_ready_rx, bridge_join_handle) = Self::spawn_bridge(socket, rx, handshake_tx);
        if let Err(e) = bridge_ready_rx.await {
            return Err(TapError::ThreadPanic(e.to_string()));
        }

        debug!("awaiting server handshake...");
        let handshake = Self::await_handshake(handshake_rx).await?;
        info!(
            "handshake successful, protocol version: {}",
            handshake.server_protocol_version
        );

        Ok(APIClient {
            server: ServerInfo {
                addr: server_addr,
                protocol_version: handshake.server_protocol_version,
            },
            conn: Connection {
                bridge_handle: bridge_join_handle,
                sender: tx,
            },
        })
    }

    async fn connect_tcp<A: ToSocketAddrs>(
        addr: A,
    ) -> TapResult<(Framed<TcpStream, LinesCodec>, String)> {
        let stream = TcpStream::connect(addr).await?;
        let addr = stream.peer_addr()?.to_string();
        let socket = Framed::new(stream, LinesCodec::new_with_max_length(1024));
        Ok((socket, addr))
    }

    fn spawn_bridge(
        socket: Framed<TcpStream, LinesCodec>,
        rx: mpsc::Receiver<Envelope>,
        handshake_tx: oneshot::Sender<Packet>,
    ) -> (oneshot::Receiver<()>, JoinHandle<()>) {
        let (ready_tx, ready_rx) = oneshot::channel::<()>();

        let join_handle = tokio::spawn(async move {
            let mut bridge = Bridge::new(socket, rx);
            bridge.listen(handshake_tx, ready_tx).await;
        });

        (ready_rx, join_handle)
    }

    async fn await_handshake(
        handshake_rx: oneshot::Receiver<Packet>,
    ) -> TapResult<HandshakePacket> {
        let packet = handshake_rx.await.map_err(|_| TapError::Disconnected)?;
        HandshakePacket::try_from(packet)
    }
}

impl APIClient {
    pub fn exit(&self) {
        self.conn.bridge_handle.abort();
    }

    pub async fn connect(&self, player_name: String) -> TapResult<()> {
        let command = connect::create_command_connect(&self.server, player_name.clone())?;

        debug!("sending connect command for player: {}", player_name);
        let packet = self.conn.send(command).await?;

        let _ = ConnectPacket::parse(&self.server, packet)?;
        info!("player {} connected successfully", player_name);

        Ok(())
    }
}
