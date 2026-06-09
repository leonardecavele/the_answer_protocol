use crate::command::{connect, Command};
use crate::envelop::Envelop;
use crate::network::Bridge;
use crate::packet::connect::ConnectPacket;
use crate::packet::handshake::HandshakePacket;
use crate::packet::Packet;
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
    sender: mpsc::Sender<Envelop>,
}

impl Connection {
    async fn send(&self, command: Command) -> io::Result<Packet> {
        let (tx, rx) = oneshot::channel::<Packet>();

        let envelop = Envelop {
            command,
            tx: Some(tx),
        };

        match self.sender.send(envelop).await {
            Ok(()) => match rx.await {
                Ok(result) => Ok(result),
                Err(e) => Err(Error::new(
                    ErrorKind::Other,
                    format!("Recv command error: {}", e),
                )),
            },
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("Send command error: {}", e),
            )),
        }
    }
}

impl APIClient {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> io::Result<APIClient> {
        let stream = TcpStream::connect(addr).await?;
        let socket = Framed::new(stream, LinesCodec::new_with_max_length(1024));
        let (tx, rx) = mpsc::channel::<Envelop>(1024);

        let server_addr: String = socket.get_ref().peer_addr()?.to_string();
        // let handshake = {
        //     let raw_line = socket
        //         .next()
        //         .await
        //         .ok_or_else(|| {
        //             Error::new(
        //                 ErrorKind::UnexpectedEof,
        //                 "Client disconnected without sending handshake",
        //             )
        //         })?
        //         .map_err(|e| {
        //             Error::new(
        //                 ErrorKind::InvalidData,
        //                 format!("Failed to read line: {}", e),
        //             )
        //         })?;
        //
        //     let packet = Packet::parse(raw_line).map_err(|e| {
        //         Error::new(
        //             ErrorKind::InvalidData,
        //             format!("Failed to retrieve server protocol version: {}", e),
        //         )
        //     })?;
        //
        //     HandshakePacket::try_from(packet).map_err(|e| {
        //         Error::new(
        //             ErrorKind::InvalidData,
        //             format!("Protocol error during handshake: {}", e),
        //         )
        //     })?
        // };

        let (handshake_tx, handshake_rx) = oneshot::channel::<Packet>();

        tokio::spawn(async move {
            let mut bridge = Bridge::new(socket, rx);
            bridge.listen(handshake_tx).await;
        });

        let handshake = match handshake_rx.await {
            Ok(packet) => HandshakePacket::try_from(packet).map_err(|e| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Protocol error during handshake: {}", e),
                )
            })?,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::ConnectionReset,
                    format!("Client disconnected without sending handshake: {}", e),
                ));
            }
        };

        let conn = Connection { sender: tx };

        Ok(APIClient {
            server: ServerInfo {
                addr: server_addr,
                protocol_version: handshake.server_protocol_version,
            },
            conn,
        })
    }

    pub async fn connect(&self, player_name: String) -> io::Result<()> {
        let command = connect::create_command_connect(&self.server, player_name)?;
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
