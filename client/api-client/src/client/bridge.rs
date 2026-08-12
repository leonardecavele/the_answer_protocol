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
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

enum Disconnection {
    ClosedByClient,
    ServerClosed,
    RequestTimedOut { command: String, timeout: Duration },
    Failed(TapError),
}

enum Flow {
    Continue,
    Stop(Disconnection),
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
    request_timeout: Duration,
}

impl Bridge {
    pub fn new(
        socket: Framed<TcpStream, LinesCodec>,
        event_sender: broadcast::Sender<ServerEvent>,
        command_receiver: Receiver<Request>,
        request_timeout: Duration,
    ) -> Bridge {
        Bridge {
            socket,
            event_sender,
            command_receiver,
            pending_request: None,
            request_timeout,
        }
    }

    pub async fn listen(&mut self, cancellation: CancellationToken) {
        info!("bridge is now listening for incoming and outgoing packets");

        let disconnection = loop {
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
                        None => Ok(Flow::Stop(Disconnection::ClosedByClient)),
                    }
                },

                _ = cancellation.cancelled() => Ok(Flow::Stop(Disconnection::ClosedByClient)),

                _ = sleep_until(expires_at), if self.pending_request.is_some() => {
                    let PendingRequest { request, timeout, .. } = self
                        .pending_request
                        .take()
                        .expect("guarded by the select! precondition");

                    let command = request.raw_command.clone();
                    let timeout_error = NetworkError::RequestTimeout {
                        command: request.raw_command,
                        timeout,
                    };
                    let _ = request.reply_to.send(Err(timeout_error.into()));

                    Ok(Flow::Stop(Disconnection::RequestTimedOut { command, timeout }))
                }
            };

            match flow {
                Ok(Flow::Continue) => {}
                Ok(Flow::Stop(disconnection)) => break disconnection,
                Err(fatal_error) => break Disconnection::Failed(fatal_error),
            }
        };

        let reason = match &disconnection {
            Disconnection::ClosedByClient => None,
            Disconnection::ServerClosed => Some("server closed the connection".to_string()),
            Disconnection::RequestTimedOut { command, timeout } => Some(format!(
                "no answer to '{}' within {}s",
                command,
                timeout.as_secs()
            )),
            Disconnection::Failed(fatal_error) => Some(fatal_error.to_string()),
        };

        match &reason {
            None => info!("connection closed by the client"),
            Some(reason) => warn!("connection lost: {}", reason),
        }

        let _ = SinkExt::<String>::close(&mut self.socket).await;

        if let Some(reason) = reason {
            let _ = self.event_sender.send(ServerEvent::ConnectionLost(reason));
        }

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
                self.fail_pending_request(NetworkError::Disconnected.into());
                return Ok(Flow::Stop(Disconnection::ServerClosed));
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
            timeout: self.request_timeout,
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
