use crate::client::event::ServerEvent;
use crate::error::{NetworkError, TapError};
use crate::protocol::request::Request;
use crate::protocol::response::{Opcode, ServerResponse};
use futures::SinkExt;
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Instant, sleep_until};
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use tracing::{debug, error, info, warn};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

enum Flow {
    Continue,
    Stop,
}

struct PendingRequest {
    request: Request,
    created_at: Instant,
    timeout: Duration,
}

impl PendingRequest {
    fn expires_at(&self) -> Instant {
        self.created_at + self.timeout
    }
}

pub struct Bridge {
    socket: Framed<TcpStream, LinesCodec>,
    event_sender: broadcast::Sender<ServerEvent>,
    command_receiver: Receiver<Request>,
    pending_request: Option<PendingRequest>,
}

impl Bridge {
    pub fn new(
        socket: Framed<TcpStream, LinesCodec>,
        event_sender: broadcast::Sender<ServerEvent>,
        command_receiver: Receiver<Request>,
    ) -> Bridge {
        Bridge {
            socket,
            event_sender,
            command_receiver,
            pending_request: None,
        }
    }

    pub async fn listen(
        &mut self,
        handshake_request: Request,
        ready_sender: tokio::sync::oneshot::Sender<()>,
    ) {
        self.pending_request = Some(PendingRequest {
            request: handshake_request,
            created_at: Instant::now(),
            timeout: HANDSHAKE_TIMEOUT,
        });
        info!("bridge is now listening for incoming and outgoing packets");

        let _ = ready_sender.send(());

        loop {
            let expires_at = self
                .pending_request
                .as_ref()
                .map(PendingRequest::expires_at)
                .unwrap_or_else(Instant::now);

            let flow = tokio::select! {
                frame = self.socket.next() => self.handle_incoming(frame).await,

                command = self.command_receiver.recv(), if self.pending_request.is_none() => {
                    match command {
                        Some(request) => self.handle_outgoing(request).await,
                        None => {
                            info!("command channel closed (client dropped), shutting down bridge");
                            Ok(Flow::Stop)
                        }
                    }
                },

                _ = sleep_until(expires_at), if self.pending_request.is_some() => {
                    let PendingRequest { request, timeout, .. } = self
                        .pending_request
                        .take()
                        .expect("guarded by the select! precondition");

                    let timeout_error = NetworkError::RequestTimeout {
                        command: request.raw_command,
                        timeout,
                    };
                    warn!("{}", timeout_error);

                    let _ = request.reply_to.send(Err(timeout_error.into()));
                    Ok(Flow::Stop)
                }
            };

            match flow {
                Ok(Flow::Continue) => {}
                Ok(Flow::Stop) => break,
                Err(fatal_error) => {
                    error!(
                        "bridge is shutting down, connection unusable: {}",
                        fatal_error
                    );
                    break;
                }
            }
        }

        let _ = self.event_sender.send(ServerEvent::ConnectionLost);
        info!("network connection closed");
    }

    async fn handle_incoming(
        &mut self,
        frame: Option<Result<String, LinesCodecError>>,
    ) -> Result<Flow, TapError> {
        let line = match frame {
            Some(Ok(line)) => line,
            Some(Err(codec_error)) => {
                self.fail_pending_request(NetworkError::Disconnected.into());
                return Err(NetworkError::Codec(codec_error).into());
            }
            None => {
                info!("server closed the connection");
                self.fail_pending_request(NetworkError::Disconnected.into());
                return Ok(Flow::Stop);
            }
        };

        debug!("recv frame: {}", line);

        let response = match ServerResponse::try_from(line) {
            Ok(response) => response,
            Err(parse_error) => {
                match self.pending_request.take() {
                    Some(PendingRequest { request, .. }) => {
                        warn!(
                            "unreadable frame while '{}' was pending: {}. \
                             Failing that command.",
                            request.raw_command, parse_error
                        );
                        let _ = request.reply_to.send(Err(parse_error.into()));
                    }
                    None => warn!("ignoring unreadable frame: {}", parse_error),
                }

                return Ok(Flow::Continue);
            }
        };

        match response.opcode {
            Opcode::Empty => {
                debug!("ignoring empty frame");
                return Ok(Flow::Continue);
            }
            Opcode::Evt => {
                let _ = self.event_sender.send(ServerEvent::from(response));
                return Ok(Flow::Continue);
            }
            Opcode::Ok | Opcode::Err => {}
        }

        match self.pending_request.take() {
            Some(PendingRequest { request, .. }) => {
                if request.reply_to.send(Ok(response)).is_err() {
                    warn!(
                        "nobody is waiting for the response to '{}' anymore, dropping it",
                        request.raw_command
                    );
                }
            }
            None => {
                error!(
                    "protocol desync: server sent '{}' while no command was pending",
                    response.raw
                );
            }
        }

        Ok(Flow::Continue)
    }

    async fn handle_outgoing(&mut self, request: Request) -> Result<Flow, TapError> {
        debug_assert!(
            self.pending_request.is_none(),
            "handle_outgoing called while a command is still pending: \
             the select! guard should have made this unreachable"
        );

        let raw_command = request.raw_command.clone();
        self.pending_request = Some(PendingRequest {
            request,
            created_at: Instant::now(),
            timeout: REQUEST_TIMEOUT,
        });

        debug!("send frame: {}", raw_command);

        if let Err(codec_error) = self.socket.send(raw_command).await {
            self.fail_pending_request(NetworkError::Disconnected.into());
            return Err(NetworkError::Codec(codec_error).into());
        }

        Ok(Flow::Continue)
    }

    fn fail_pending_request(&mut self, error: TapError) {
        if let Some(PendingRequest { request, .. }) = self.pending_request.take() {
            let _ = request.reply_to.send(Err(error));
        }
    }
}
