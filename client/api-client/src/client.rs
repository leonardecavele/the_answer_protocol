use crate::error::{TapError, TapResult};
use crate::network::bridge::Bridge;
use crate::protocol::command::connect::{ConnectCommand, ConnectServerResponseData};
use crate::protocol::command::{Command, CommandResult};
use crate::protocol::handshake::HandshakeServerResponse;
use crate::protocol::request::Request;
use crate::protocol::response::{ServerResponse, ServerResponseOpcode};
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
    bridge_thread: JoinHandle<()>,
    request_transmitter: mpsc::Sender<Request>,
}

impl APIClient {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> TapResult<APIClient> {
        let (socket, server_addr) = Self::connect_tcp(addr).await?;
        info!("successfully connected to TCP socket at {}", server_addr);

        let (request_transmitter, request_receiver) = mpsc::channel::<Request>(1024);
        let (handshake_request, handshake_receiver) = Request::handshake();

        let bridge_thread = Self::start_bridge(socket, handshake_request, request_receiver).await?;

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
            conn: Connection {
                bridge_thread,
                request_transmitter,
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

    async fn start_bridge(
        socket: Framed<TcpStream, LinesCodec>,
        handshake_request: Request,
        command_receiver: mpsc::Receiver<Request>,
    ) -> TapResult<JoinHandle<()>> {
        let (ready_transmitter, ready_receiver) = oneshot::channel::<()>();

        let bridge_thread = tokio::spawn(async move {
            let mut bridge = Bridge::new(socket, command_receiver);
            bridge.listen(handshake_request, ready_transmitter).await;
        });

        ready_receiver
            .await
            .map_err(|e| TapError::ThreadPanic(e.to_string()))?;

        Ok(bridge_thread)
    }

    async fn await_handshake(
        handshake_receiver: oneshot::Receiver<ServerResponse>,
    ) -> TapResult<HandshakeServerResponse> {
        let response = handshake_receiver
            .await
            .map_err(|_| TapError::Disconnected)?;
        HandshakeServerResponse::try_from(response)
    }

    async fn request<C: Command>(&self, command: C) -> TapResult<CommandResult<C::ResponseData>> {
        let payload = command.create_command(&self.server)?;

        let (request, response_receiver) = Request::new(payload);

        self.conn
            .request_transmitter
            .send(request)
            .await
            .map_err(|e| TapError::Channel(format!("[client] send request error: {}", e)))?;

        let response = response_receiver
            .await
            .map_err(|e| TapError::Channel(format!("[client] recv request error: {}", e)))?;

        if response.opcode == ServerResponseOpcode::Ok {
            command.parse_response_ok(&self.server, response)
        } else {
            Ok(CommandResult::error_from_response(response))
        }
    }
}

impl APIClient {
    pub async fn connect(
        &self,
        player_name: String,
    ) -> TapResult<CommandResult<ConnectServerResponseData>> {
        debug!("sending connect request for player: {}", player_name);

        let response = self
            .request(ConnectCommand {
                player_name: player_name.clone(),
            })
            .await?;

        Ok(response)
    }

    pub fn close(&self) {
        self.conn.bridge_thread.abort();
    }
}
