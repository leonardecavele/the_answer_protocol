use crate::protocol::envelope::Envelope;
use crate::protocol::packet::Packet;
use futures::stream::StreamExt;
use futures::SinkExt;
use std::io::{Error, ErrorKind, Result};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};

pub struct Bridge {
    socket: Framed<TcpStream, LinesCodec>,
    rx: Receiver<Envelope>,
    pending_request: Option<tokio::sync::oneshot::Sender<Packet>>,
}

impl Bridge {
    pub fn new(
        socket: Framed<TcpStream, LinesCodec>,
        rx: Receiver<Envelope>,
    ) -> Bridge {
        Bridge {
            socket,
            rx,
            pending_request: None,
        }
    }

    pub async fn listen(
        &mut self,
        handshake_tx: tokio::sync::oneshot::Sender<Packet>,
    ) -> () {
        self.pending_request = Some(handshake_tx);

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
                            println!("[network incoming] error during handling incoming frame: {}", e);
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
                            println!("[network outgoing] error during handling incoming frame: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        println!("[network] connection closed");
    }

    async fn handle_incoming(
        &mut self,
        frame: Option<core::result::Result<String, LinesCodecError>>,
    ) -> Result<bool> {
        if frame.is_none() {
            return Ok(false);
        }

        match frame.unwrap() {
            Ok(line) => {
                println!("[network bridge] receive response: {}", line);

                match Packet::parse(line) {
                    Ok(packet) => {
                        if let Some(tx) = self.pending_request.take() {
                            tx.send(packet).and(Ok(true)).map_err(|pkt| {
                                Error::new(
                                    ErrorKind::BrokenPipe,
                                    format!(
                                        "[network incoming] lost packet {:?}",
                                        pkt
                                    ),
                                )
                            })
                        } else {
                            Ok(true)
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "[network bridge] error parsing packet: {}",
                            e
                        );
                        Ok(true)
                    }
                }
            }
            Err(e) => {
                eprintln!("[network bridge] error reading from socket: {}", e);
                Ok(true)
            }
        }
    }

    async fn handle_outgoing(
        &mut self,
        request: Option<Envelope>,
    ) -> Result<bool> {
        match request {
            Some(envelope) => {
                if !self.pending_request.is_none() {
                    eprintln!(
                        "[network outgoing] waiting for another request. command dropped"
                    );
                    return Ok(true);
                }

                self.pending_request = envelope.tx;
                println!(
                    "[network outgoing] send command: {}",
                    envelope.command.command
                );

                if let Err(e) = self.socket.send(envelope.command.command).await
                {
                    eprintln!(
                        "[network outgoing] error sending to socket: {}",
                        e
                    );
                }

                Ok(true)
            }
            None => {
                eprintln!("[network outgoing] no data to send");
                Ok(true)
            }
        }
    }
}
