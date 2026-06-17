use crate::client::event::ServerEvent;
use crate::error::{InternalError, NetworkError, TapError};
use crate::protocol::request::Request;
use crate::protocol::response::{Opcode, ServerResponse};
use futures::stream::StreamExt;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use tracing::{debug, error, info, warn};

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
                            error!("{}", e);
                            break;
                        }
                    }
                },
                // send response to the server
                request_opt = self.command_receiver.recv(), if self.pending_request.is_none() => {
                    let request = match request_opt {
                        Some(req) => req,
                        None => {
                            info!("command channel closed (client dropped), shutting down bridge");
                            break;
                        }
                    };

                    match self.handle_outgoing(request).await {
                        Ok(can_continue) => {
                            if !can_continue {
                                break;
                            }
                        },
                        Err(e) => {
                            error!("{}", e);
                            break;
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
    ) -> Result<bool, TapError> {
        let frame_result = match frame {
            Some(f) => f,
            None => return Ok(false),
        };

        match frame_result {
            Ok(line) => {
                debug!("receive response: {}", line);
                let response = ServerResponse::try_from(line)?;

                if response.opcode == Opcode::Evt {
                    let event = ServerEvent::from(response);
                    let _ = self.event_transmitter.send(event).map_err(|_| {
                        InternalError::ChannelPanic(
                            "event channel is closed, nowhere to send the event".to_string(),
                        )
                    })?;

                    return Ok(true);
                }

                if let Some(request) = self.pending_request.take() {
                    let result = request.reply_to.send(response).map(|_| true).map_err(|_| {
                        InternalError::ChannelPanic(
                            "the requester dropped the receiver \
                                        before getting the response"
                                .to_string(),
                        )
                    })?;

                    Ok(result)
                } else {
                    error!(
                        "received unexpected response \
                                from server while no request was pending"
                    );

                    Ok(true)
                }
            }
            Err(codec_error) => {
                error!("fatal network read error: {}", codec_error);
                Err(NetworkError::Codec(codec_error).into())
            }
        }
    }

    async fn handle_outgoing(&mut self, request: Request) -> Result<bool, TapError> {
        if self.pending_request.is_some() {
            warn!(
                "a request is already pending. Dropping the new command: '{}'",
                request.raw_command
            );
            return Ok(true);
        }

        let raw_command = request.raw_command.clone();
        self.pending_request = Some(request);

        debug!("send request: '{}'", raw_command.clone());

        if let Err(codec_error) = self.socket.send(raw_command).await {
            error!("fatal network write error: {}", codec_error);
            return Err(NetworkError::Codec(codec_error).into());
        }

        Ok(true)
    }

}
