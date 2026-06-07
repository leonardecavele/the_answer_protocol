pub mod network_bridge;
pub mod packet;

use crate::packet::greeting::GreetingPacket;
use crate::packet::FromPacket;
use futures::stream::StreamExt;
use network_bridge::NetworkBridge;
use packet::Packet;
use std::fmt::{Display, Write};
use std::io::{Error, ErrorKind, Result};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LinesCodec};

pub struct ServerInfo {
    pub addr: String,
    pub protocol_version: String,
}

pub struct APIClient {
    pub server: ServerInfo,
    pub sender: mpsc::Sender<String>,
}

impl APIClient {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> Result<APIClient> {
        let stream = TcpStream::connect(addr).await?;
        let mut socket =
            Framed::new(stream, LinesCodec::new_with_max_length(1024));

        let server_addr: String = socket.get_ref().peer_addr()?.to_string();
        let greeting = {
            let raw_line = socket
                .next()
                .await
                .ok_or_else(|| {
                    Error::new(
                        ErrorKind::UnexpectedEof,
                        "Client disconnected without sending greeting",
                    )
                })?
                .map_err(|e| {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("Failed to read line: {}", e),
                    )
                })?;

            let frame = Packet::new(raw_line).map_err(|e| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Failed to retrieve server protocol version: {}",
                        e
                    ),
                )
            })?;

            GreetingPacket::parse(frame).map_err(|e| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Protocol error during greeting: {}", e),
                )
            })?
        };

        let (tx, rx) = mpsc::channel::<String>(1024);

        let mut bridge = NetworkBridge::new(socket, rx);
        tokio::spawn(async move {
            bridge.listen().await;
        });

        Ok(APIClient {
            server: ServerInfo {
                addr: server_addr,
                protocol_version: greeting.server_protocol_version,
            },
            sender: tx,
        })
    }
}
