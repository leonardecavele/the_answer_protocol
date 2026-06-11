use crate::error::{TapError, TapResult};
use crate::network::bridge::Bridge;
use crate::protocol::command::connect::{ConnectCommand, ConnectResponse};
use crate::protocol::command::look::{LookCommand, LookResponse};
use crate::protocol::command::quit::{QuitCommand, QuitResponse};
use crate::protocol::command::{Command, CommandResult, CreateCommandResult};
use crate::protocol::handshake::HandshakeResponse;
use crate::protocol::request::Request;
use crate::protocol::response::{ServerResponse, Opcode};
use std::fmt::Display;
use std::process::exit;
use std::time::Duration;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{debug, info, warn};

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
    handler: JoinHandle<()>,
    sender: mpsc::Sender<Request>,
    event_manager: EventManager,
}

struct EventManager {
    sender: broadcast::Sender<ServerResponse>,
    handlers: Vec<JoinHandle<()>>,
}

impl APIClient {
    pub async fn new<A>(addr: A) -> TapResult<APIClient>
    where
        A: ToSocketAddrs + Clone + Display,
    {
        let (socket, server_addr) = Self::connect_tcp(addr).await?;
        info!("successfully connected to TCP socket at {}", server_addr);

        let (request_sender, request_receiver) = mpsc::channel::<Request>(2048);
        let (event_sender, _) = broadcast::channel::<ServerResponse>(2048);
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
            conn: Connection {
                handler: bridge_handler,
                sender: request_sender,
                event_manager: EventManager {
                    sender: event_sender,
                    handlers: vec![],
                },
            },
        })
    }

    async fn connect_tcp<A: ToSocketAddrs + Clone + Display>(
        addr: A,
    ) -> TapResult<(Framed<TcpStream, LinesCodec>, String)>
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
                Ok(Err(e)) => {
                    if attempt >= max_attempt {
                        return Err(TapError::Io(e));
                    }
                    info!(
                        "failed to connect to {}, retrying in {} milliseconds..",
                        addr, timeout_before_retry
                    );
                }
                Err(_) => {
                    if attempt >= max_attempt {
                        return Err(TapError::Io(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "connection timed out",
                        )));
                    }
                    info!(
                        "connection to {} timed out, retrying in {} milliseconds..",
                        addr, timeout_before_retry
                    );
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(timeout_before_retry)).await;
        }

        Err(TapError::Disconnected)
    }

    async fn start_bridge(
        socket: Framed<TcpStream, LinesCodec>,
        handshake_request: Request,
        event_sender: broadcast::Sender<ServerResponse>,
        command_receiver: mpsc::Receiver<Request>,
    ) -> TapResult<JoinHandle<()>> {
        let (ready_sender, ready_receiver) = oneshot::channel::<()>();

        let bridge_handler = tokio::spawn(async move {
            let mut bridge = Bridge::new(socket, event_sender, command_receiver);
            bridge.listen(handshake_request, ready_sender).await;
        });

        ready_receiver
            .await
            .map_err(|e| TapError::ThreadPanic(e.to_string()))?;

        Ok(bridge_handler)
    }

    async fn await_handshake(
        handshake_receiver: oneshot::Receiver<ServerResponse>,
    ) -> TapResult<HandshakeResponse> {
        let response = handshake_receiver
            .await
            .map_err(|_| TapError::Disconnected)?;
        HandshakeResponse::try_from(response)
    }

    async fn request<C: Command>(&self, command: C) -> TapResult<CommandResult<C::ResponseData>> {
        let create_command_result = command.create_command(&self.server);

        match create_command_result {
            CreateCommandResult::Success { raw_command } => {
                let (request, response_receiver) = Request::new(raw_command);

                self.conn
                    .sender
                    .send(request)
                    .await
                    .map_err(|e| {
                        TapError::Channel(format!("[client] send request error: {}", e))
                    })?;

                let response = response_receiver.await.map_err(|e| {
                    TapError::Channel(format!("[client] recv request error: {}", e))
                })?;

                if response.opcode == Opcode::Ok {
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
    pub fn on_event(&mut self, handler: fn(ServerResponse) -> ()) {
        let mut subscriber = self.conn.event_manager.sender.subscribe();

        self.conn
            .event_manager
            .handlers
            .push(tokio::spawn(async move {
                loop {
                    match subscriber.recv().await {
                        Ok(event) => handler(event),
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("lag.. {} events dropped", skipped);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            }))
    }

    pub async fn connect(
        &self,
        player_name: String,
    ) -> TapResult<CommandResult<ConnectResponse>> {
        debug!("sending connect request for player: {}", player_name);

        let response = self.request(ConnectCommand { player_name }).await?;

        Ok(response)
    }

    pub async fn look(&self) -> TapResult<CommandResult<LookResponse>> {
        debug!("sending look request");

        let response = self.request(LookCommand).await?;

        Ok(response)
    }

    pub async fn quit(self) {
        debug!("sending quit request");

        let _ = self.request(QuitCommand).await;
        self.close()
    }

    pub fn close(self) {
        self.conn.handler.abort();

        for event_subscriber in self.conn.event_manager.handlers.iter() {
            event_subscriber.abort()
        }

        info!("client connection terminated");
    }
}
