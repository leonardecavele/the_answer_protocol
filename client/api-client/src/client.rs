use crate::error::{TapError, TapResult};
use crate::network::bridge::Bridge;
use crate::protocol::command::connect::{ConnectCommand, ConnectServerResponseData};
use crate::protocol::command::look::{LookCommand, LookServerResponseData};
use crate::protocol::command::{Command, CommandResult, CreateCommandResult};
use crate::protocol::handshake::HandshakeServerResponse;
use crate::protocol::request::Request;
use crate::protocol::response::{ServerResponse, ServerResponseOpcode};
use std::fmt::Display;
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
    pub async fn new<A>(addr: A) -> TapResult<APIClient>
    where
        A: ToSocketAddrs + Clone + Display,
    {
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

    async fn connect_tcp<A: ToSocketAddrs + Clone + Display>(
        addr: A,
    ) -> TapResult<(Framed<TcpStream, LinesCodec>, String)>
    where
        A: ToSocketAddrs + Clone + Display,
    {
        let max_attempt: u32 = 3;
        let timeout_before_retry: u64 = 3;

        for attempt in 1..=max_attempt {
            match TcpStream::connect(addr.clone()).await {
                Ok(stream) => {
                    let peer_addr = stream.peer_addr()?.to_string();
                    let socket = Framed::new(stream, LinesCodec::new_with_max_length(1024));
                    return Ok((socket, peer_addr));
                }
                Err(e) => {
                    if attempt > max_attempt {
                        return Err(TapError::Io(e));
                    }
                    info!(
                        "({}/{}) failed to connect to {}, retriying in {} seconds..",
                        attempt, max_attempt, addr, timeout_before_retry
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(timeout_before_retry))
                        .await;
                }
            }
        }

        Err(TapError::Disconnected)
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
        let create_command_result = command.create_command(&self.server);

        match create_command_result {
            CreateCommandResult::Success { raw_command } => {
                let (request, response_receiver) = Request::new(raw_command);

                self.conn
                    .request_transmitter
                    .send(request)
                    .await
                    .map_err(|e| {
                        TapError::Channel(format!("[client] send request error: {}", e))
                    })?;

                let response = response_receiver.await.map_err(|e| {
                    TapError::Channel(format!("[client] recv request error: {}", e))
                })?;

                if response.opcode == ServerResponseOpcode::Ok {
                    Ok(command.parse_response_ok(&self.server, response))
                } else {
                    Ok(CommandResult::error_from_response(response))
                }
            }
            CreateCommandResult::Error { message } => Ok(CommandResult::Error { message }),
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

    pub async fn look(&self) -> TapResult<CommandResult<LookServerResponseData>> {
        debug!("sending look request");

        let response = self.request(LookCommand).await?;

        Ok(response)
    }

    pub fn close(&self) {
        self.conn.bridge_thread.abort();
        info!("close client connection");
    }
}
