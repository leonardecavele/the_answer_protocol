use crate::envelop::Envelop;
use crate::packet::Packet;
use futures::SinkExt;
use futures::stream::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::codec::{Framed, LinesCodec};

pub struct Bridge {
    socket: Framed<TcpStream, LinesCodec>,
    rx: Receiver<Envelop>,
}

impl Bridge {
    pub fn new(socket: Framed<TcpStream, LinesCodec>, rx: Receiver<Envelop>) -> Bridge {
        Bridge { socket, rx }
    }

    pub async fn listen(&mut self, handshake_tx: tokio::sync::oneshot::Sender<Packet>) -> () {
        let mut forward_tx: Option<tokio::sync::oneshot::Sender<Packet>> = Some(handshake_tx);

        loop {
            tokio::select! {
                // receive packet from the server
                frame = self.socket.next() => {
                    match frame {
                        Some(Ok(line)) => {
                            println!("[network bridge] receive response: {}", line);
                            match Packet::parse(line) {
                                Ok(packet) => {
                                    if let Some(tx) = forward_tx {
                                        let _ = tx.send(packet);
                                        forward_tx = None;
                                    }
                                },
                                Err(e) => {
                                    eprintln!("[network bridge] error parsing packet: {}", e);
                                    break
                                }
                            }
                        },
                        Some(Err(e)) => {
                            eprintln!("[network bridge] error reading from socket: {}", e);
                            break;
                        },
                        None => {
                            println!("[network bridge] socket closed");
                            break;
                        }
                    }
                },
                // send packet to the server
                data = self.rx.recv() => {
                    match data {
                        Some(envelop) => {
                            if !forward_tx.is_none() {
                                eprintln!("[network bridge] waiting for another request. command dropped.");
                                return;
                            }
                            forward_tx = envelop.tx;
                            println!("[network bridge] send command: {}", envelop.command.command);
                            if let Err(e) = self.socket.send(envelop.command.command).await {
                                eprintln!("[network bridge] error sending to socket: {}", e);
                            }
                        },
                        None => {
                            eprintln!("[network bridge] No data to send");
                        }
                    }
                }
            }
        }

        println!("[network bridge] connection closed");
    }
}
