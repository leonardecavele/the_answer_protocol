use crate::client::ConnectionState;
use crate::client::event::ServerEvent;
use crate::error::{NetworkError, TapError};
use crate::protocol::frame::{Frame, FrameDirection};
use crate::protocol::request::Request;
use crate::protocol::response::{Opcode, ServerResponse};
use futures::SinkExt;
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{broadcast, watch};
use tokio::task::JoinHandle;
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

impl Disconnection {
    fn reason(self) -> Option<String> {
        match self {
            Disconnection::ClosedByClient => None,
            Disconnection::ServerClosed => Some("server closed the connection".to_string()),
            Disconnection::RequestTimedOut { command, timeout } => Some(format!(
                "no answer to '{}' within {}s",
                command,
                timeout.as_secs()
            )),
            Disconnection::Failed(fatal_error) => Some(fatal_error.to_string()),
        }
    }
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

pub(crate) struct BridgeChannels {
    pub state: watch::Sender<ConnectionState>,
    pub frames: broadcast::Sender<Frame>,
    pub events: broadcast::Sender<ServerEvent>,
    pub requests: Receiver<Request>,
}

pub(crate) struct Bridge {
    socket: Framed<TcpStream, LinesCodec>,
    channels: BridgeChannels,
    pending_request: Option<PendingRequest>,
    request_timeout: Duration,
}

impl Bridge {
    pub(crate) fn start(
        socket: Framed<TcpStream, LinesCodec>,
        channels: BridgeChannels,
        cancellation: CancellationToken,
        request_timeout: Duration,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut bridge = Bridge {
                socket,
                channels,
                pending_request: None,
                request_timeout,
            };
            let disconnection = bridge.run(cancellation).await;
            bridge.shutdown(disconnection).await;
        })
    }

    async fn run(&mut self, cancellation: CancellationToken) -> Disconnection {
        info!("bridge is now listening for incoming and outgoing packets");

        loop {
            let expires_at = self
                .pending_request
                .as_ref()
                .map(PendingRequest::expires_at)
                .unwrap_or_else(Instant::now);

            let flow = tokio::select! {
                frame = self.socket.next() =>
                    self.incoming_frame(frame).await,

                request_opt = self.channels.requests.recv(), if self.pending_request.is_none() =>
                    self.outgoing_request(request_opt).await,

                _ = sleep_until(expires_at), if self.pending_request.is_some() =>
                    self.request_timed_out(),

                _ = cancellation.cancelled() =>
                    Ok(Flow::Stop(Disconnection::ClosedByClient))
            };

            match flow {
                Ok(Flow::Continue) => {}
                Ok(Flow::Stop(disconnection)) => break disconnection,
                Err(fatal_error) => break Disconnection::Failed(fatal_error),
            }
        }
    }

    async fn shutdown(&mut self, disconnection: Disconnection) {
        let state = match disconnection.reason() {
            None => {
                info!("connection closed by the client");
                ConnectionState::Closed
            }
            Some(reason) => {
                warn!("connection lost: {}", reason);
                ConnectionState::Lost(reason)
            }
        };

        let _ = SinkExt::<String>::close(&mut self.socket).await;
        let _ = self.channels.state.send(state);

        info!("network connection closed");
    }

    async fn incoming_frame(
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
        let _ = self.channels.frames.send(Frame {
            direction: FrameDirection::Received,
            line: line.clone(),
        });

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
            }
            Opcode::Evt => {
                let _ = self.channels.events.send(ServerEvent::from(response));
            }
            Opcode::Ok | Opcode::Err => self.answer_pending(response),
        }

        Ok(Flow::Continue)
    }

    async fn outgoing_request(&mut self, request_opt: Option<Request>) -> Result<Flow, TapError> {
        let request = match request_opt {
            Some(request) => request,
            None => {
                return Ok(Flow::Stop(Disconnection::ClosedByClient));
            }
        };

        debug_assert!(
            self.pending_request.is_none(),
            "handle_outgoing called while a command is still pending: \
             the select! guard should have made this unreachable"
        );

        let raw_command = request.raw_command.clone();

        debug!("send frame: {}", raw_command);
        let _ = self.channels.frames.send(Frame {
            direction: FrameDirection::Sent,
            line: raw_command.clone(),
        });

        self.pending_request = Some(PendingRequest {
            request,
            created_at: Instant::now(),
            timeout: self.request_timeout,
        });

        if let Err(codec_error) = self.socket.send(raw_command).await {
            self.fail_pending_request(NetworkError::Disconnected.into());
            return Err(NetworkError::Codec(codec_error).into());
        }

        Ok(Flow::Continue)
    }

    fn answer_pending(&mut self, response: ServerResponse) {
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
    }

    fn request_timed_out(&mut self) -> Result<Flow, TapError> {
        let PendingRequest {
            request, timeout, ..
        } = self
            .pending_request
            .take()
            .expect("guarded by the select! precondition");

        let command = request.raw_command.clone();
        let timeout_error = NetworkError::RequestTimeout {
            command: request.raw_command,
            timeout,
        };
        let _ = request.reply_to.send(Err(timeout_error.into()));

        Ok(Flow::Stop(Disconnection::RequestTimedOut {
            command,
            timeout,
        }))
    }

    fn fail_pending_request(&mut self, error: TapError) {
        if let Some(PendingRequest { request, .. }) = self.pending_request.take() {
            let _ = request.reply_to.send(Err(error));
        }
    }
}
