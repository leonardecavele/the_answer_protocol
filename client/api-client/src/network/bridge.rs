use crate::error::{TapError, TapResult};
use crate::protocol::request::Request;
use crate::protocol::response::{ServerResponse, Opcode};
use futures::SinkExt;
use futures::stream::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use tracing::{debug, error, info, warn};

pub struct Bridge {
    socket: Framed<TcpStream, LinesCodec>,
    event_transmitter: broadcast::Sender<ServerResponse>,
    command_receiver: Receiver<Request>,
    pending_request: Option<Request>,
}

impl Bridge {
    pub fn new(
        socket: Framed<TcpStream, LinesCodec>,
        event_transmitter: broadcast::Sender<ServerResponse>,
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
    ) -> () {
        self.pending_request = Some(handshake_request);
        info!("bridge is now listening for incoming and outgoing packets");

        let _ = ready_transmitter.send(());

        loop {
            tokio::select! {
                // receive response from the server
                frame = self.socket.next() => {
                    match self.handle_incoming(frame).await {
                        Ok(can_continue) => {
                            if !can_continue {
                                break;
                            }
                        },
                        Err(e) => {
                            error!("error handling incoming frame: {}", e);
                        }
                    }
                },
                // send response to the server
                request = self.command_receiver.recv(), if self.pending_request.is_none() => {
                    let request = request.unwrap();
                    match self.handle_outgoing(request).await {
                        Ok(can_continue) => {
                            if !can_continue {
                                break;
                            }
                        },
                        Err(e) => {
                            error!("error handling outgoing frame: {}", e);
                        }
                    }
                }
            }
        }

        info!("network connection closed");
    }

    async fn handle_incoming(
        &mut self,
        frame: Option<Result<String, LinesCodecError>>,
    ) -> TapResult<bool> {
        if frame.is_none() {
            return Ok(false);
        }

        match frame.unwrap() {
            Ok(line) => {
                debug!("receive response: {}", line);

                match ServerResponse::try_from(line) {
                    Ok(response) => {
                        if response.opcode == Opcode::Evt {
                            let _ = self.event_transmitter.send(response).map_err(|e| {
                                TapError::Channel(format!("failed to forward event: {:?}", e))
                            })?;
                            return Ok(true);
                        }

                        if let Some(request) = self.pending_request.take() {
                            request
                                .reply_to
                                .send(response)
                                .and(Ok(true))
                                .map_err(|srv_response| {
                                    TapError::Channel(format!(
                                        "internal lost response {:?}",
                                        srv_response
                                    ))
                                })
                        } else {
                            Ok(true)
                        }
                    }
                    Err(e) => {
                        error!("error parsing response: {}", e);
                        Ok(true)
                    }
                }
            }
            Err(e) => {
                error!("error reading from socket: {}", e);
                Ok(true)
            }
        }
    }

    async fn handle_outgoing(&mut self, request: Request) -> TapResult<bool> {
        if !self.pending_request.is_none() {
            warn!("waiting for another request. Command dropped");
            return Ok(true);
        }

        let command = request.command.clone();

        self.pending_request = Some(request);
        debug!("send request: '{}'", command.clone());

        if let Err(e) = self.socket.send(command).await {
            error!("error sending to socket: {}", e);
        }

        Ok(true)
    }
}
