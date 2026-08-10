use crate::client::event::ServerEvent;
use crate::error::{NetworkError, TapError};
use crate::protocol::request::Request;
use crate::protocol::response::{Opcode, ServerResponse};
use futures::SinkExt;
use futures::stream::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use tracing::{debug, error, info, warn};

enum Flow {
    Continue,
    Stop,
}

pub struct Bridge {
    socket: Framed<TcpStream, LinesCodec>,
    event_transmitter: broadcast::Sender<ServerEvent>,
    command_receiver: Receiver<Request>,
    pending_request: Option<Request>,
}

impl Bridge {
    pub fn new(
        socket: Framed<TcpStream, LinesCodec>,
        event_transmitter: broadcast::Sender<ServerEvent>,
        command_receiver: Receiver<Request>,
    ) -> Bridge {
        Bridge {
            socket,
            event_transmitter,
            command_receiver,
            pending_request: None,
        }
    }

    pub async fn listen(
        &mut self,
        handshake_request: Request,
        ready_transmitter: tokio::sync::oneshot::Sender<()>,
    ) {
        self.pending_request = Some(handshake_request);
        info!("bridge is now listening for incoming and outgoing packets");

        let _ = ready_transmitter.send(());

        loop {
            let flow = tokio::select! {
                frame = self.socket.next() => self.handle_incoming(frame).await,
                request = self.command_receiver.recv(), if self.pending_request.is_none() => {
                    match request {
                        Some(request) => self.handle_outgoing(request).await,
                        None => {
                            info!("command channel closed (client dropped), shutting down bridge");
                            Ok(Flow::Stop)
                        }
                    }
                }
            };

            match flow {
                Ok(Flow::Continue) => {}
                Ok(Flow::Stop) => break,
                Err(fatal) => {
                    error!("bridge is shutting down, connection unusable: {}", fatal);
                    break;
                }
            }
        }

        let _ = self.event_transmitter.send(ServerEvent::ConnectionLost);
        info!("network connection closed");
    }

    async fn handle_incoming(
        &mut self,
        frame: Option<Result<String, LinesCodecError>>,
    ) -> Result<Flow, TapError> {
        let line = match frame {
            Some(Ok(line)) => line,
            Some(Err(codec_error)) => return Err(NetworkError::Codec(codec_error).into()),
            None => {
                info!("server closed the connection");
                return Ok(Flow::Stop);
            }
        };

        debug!("recv frame: {}", line);

        let response = match ServerResponse::try_from(line) {
            Ok(response) => response,
            Err(parse_error) => {
                match self.pending_request.take() {
                    Some(request) => warn!(
                        "unreadable frame while '{}' was pending: {}. \
                         Failing that command.",
                        request.raw_command, parse_error
                    ),
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
                let _ = self.event_transmitter.send(ServerEvent::from(response));
                return Ok(Flow::Continue);
            }
            Opcode::Ok | Opcode::Err => {}
        }

        match self.pending_request.take() {
            Some(Request {
                raw_command,
                reply_to,
            }) => {
                if reply_to.send(response).is_err() {
                    warn!(
                        "nobody is waiting for the response to '{}' anymore, dropping it",
                        raw_command
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
        let raw_command = request.raw_command.clone();
        self.pending_request = Some(request);

        debug!("send frame: {}", raw_command);

        if let Err(codec_error) = self.socket.send(raw_command).await {
            return Err(NetworkError::Codec(codec_error).into());
        }

        Ok(Flow::Continue)
    }
}
