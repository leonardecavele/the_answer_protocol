use crate::error::{TapError, TapResult};
use crate::protocol::envelope::Envelope;
use crate::protocol::packet::Packet;
use futures::SinkExt;
use futures::stream::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use tracing::{debug, error, info, warn};

pub struct Bridge {
    socket: Framed<TcpStream, LinesCodec>,
    rx: Receiver<Envelope>,
    pending_request: Option<tokio::sync::oneshot::Sender<Packet>>,
}

impl Bridge {
    pub fn new(socket: Framed<TcpStream, LinesCodec>, rx: Receiver<Envelope>) -> Bridge {
        Bridge {
            socket,
            rx,
            pending_request: None,
        }
    }

    pub async fn listen(
        &mut self,
        handshake_tx: tokio::sync::oneshot::Sender<Packet>,
        ready_tx: tokio::sync::oneshot::Sender<()>,
    ) -> () {
        self.pending_request = Some(handshake_tx);
        info!("bridge is now listening for incoming and outgoing packets");

        let _ = ready_tx.send(());

        loop {
            tokio::select! {
                // receive packet from the server
                frame = self.socket.next() => {
                    match self.handle_incoming(frame).await {
                        Ok(can_continue) => {
                            if !can_continue {
                                break;
                            }
                        },
                        Err(e) => {
                            error!("error handling incoming frame: {}", e);
                            break;
                        }
                    }
                },
                // send packet to the server
                request = self.rx.recv() => {
                    match self.handle_outgoing(request).await {
                        Ok(can_continue) => {
                            if !can_continue {
                                break;
                            }
                        },
                        Err(e) => {
                            error!("error handling outgoing frame: {}", e);
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
    ) -> TapResult<bool> {
        if frame.is_none() {
            return Ok(false);
        }

        match frame.unwrap() {
            Ok(line) => {
                debug!("receive response: {}", line);

                match Packet::try_from(line) {
                    Ok(packet) => {
                        if let Some(tx) = self.pending_request.take() {
                            tx.send(packet).and(Ok(true)).map_err(|pkt| {
                                TapError::Channel(format!("internal lost packet {:?}", pkt))
                            })
                        } else {
                            Ok(true)
                        }
                    }
                    Err(e) => {
                        error!("error parsing packet: {}", e);
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

    async fn handle_outgoing(&mut self, request: Option<Envelope>) -> TapResult<bool> {
        match request {
            Some(envelope) => {
                if !self.pending_request.is_none() {
                    warn!("waiting for another request. Command dropped");
                    return Ok(true);
                }

                self.pending_request = envelope.tx;
                debug!("send command: {}", envelope.command.command);

                if let Err(e) = self.socket.send(envelope.command.command).await {
                    error!("error sending to socket: {}", e);
                }

                Ok(true)
            }
            None => {
                warn!("no data to send");
                Ok(true)
            }
        }
    }
}
