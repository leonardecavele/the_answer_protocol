use crate::network::bridge::Bridge;
use crate::protocol::command::{connect, Command};
use crate::protocol::envelope::Envelope;
use crate::protocol::packet::connect::ConnectPacket;
use crate::protocol::packet::handshake::HandshakePacket;
use crate::protocol::packet::Packet;
use std::io;
use std::io::{Error, ErrorKind};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::{mpsc, oneshot};
use tokio_util::codec::{Framed, LinesCodec};

pub struct ServerInfo {
    pub addr: String,
    pub protocol_version: u32,
}

pub struct APIClient {
    pub server: ServerInfo,
    conn: Connection,
}

struct Connection {
    sender: mpsc::Sender<Envelope>,
}

impl Connection {
    async fn send(&self, command: Command) -> io::Result<Packet> {
        let (tx, rx) = oneshot::channel::<Packet>();

        let envelope = Envelope {
            command,
            tx: Some(tx),
        };

        self.sender.send(envelope).await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("[client] Send command error: {}", e),
            )
        })?;

        rx.await.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("[client] Recv command error: {}", e),
            )
        })
    }
}

impl APIClient {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> io::Result<APIClient> {
        let (socket, server_addr) = Self::connect_tcp(addr).await?;
        let (tx, rx) = mpsc::channel::<Envelope>(1024);
        let (handshake_tx, handshake_rx) = oneshot::channel::<Packet>();

        Self::spawn_bridge(socket, rx, handshake_tx);

        let handshake = Self::await_handshake(handshake_rx).await?;

        Ok(APIClient {
            server: ServerInfo {
                addr: server_addr,
                protocol_version: handshake.server_protocol_version,
            },
            conn: Connection { sender: tx },
        })
    }

    async fn connect_tcp<A: ToSocketAddrs>(
        addr: A,
    ) -> io::Result<(Framed<TcpStream, LinesCodec>, String)> {
        let stream = TcpStream::connect(addr).await?;
        let addr = stream.peer_addr()?.to_string();
        let socket = Framed::new(stream, LinesCodec::new_with_max_length(1024));
        Ok((socket, addr))
    }

    fn spawn_bridge(
        socket: Framed<TcpStream, LinesCodec>,
        rx: mpsc::Receiver<Envelope>,
        handshake_tx: oneshot::Sender<Packet>,
    ) {
        tokio::spawn(async move {
            let mut bridge = Bridge::new(socket, rx);
            bridge.listen(handshake_tx).await;
        });
    }

    async fn await_handshake(
        handshake_rx: oneshot::Receiver<Packet>,
    ) -> io::Result<HandshakePacket> {
        let packet = handshake_rx.await.map_err(|e| {
            Error::new(
                ErrorKind::ConnectionReset,
                format!("Client disconnected without sending handshake: {}", e),
            )
        })?;

        HandshakePacket::try_from(packet).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Protocol error during handshake: {}", e),
            )
        })
    }
}

impl APIClient {
    pub async fn connect(&self, player_name: String) -> io::Result<()> {
        let command =
            connect::create_command_connect(&self.server, player_name)?;
        let packet = self.conn.send(command).await?;

        let _ = ConnectPacket::try_from(packet).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Protocol error during connect: {}", e),
            )
        })?;

        Ok(())
    }
}
