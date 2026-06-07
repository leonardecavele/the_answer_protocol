use futures::stream::StreamExt;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio_util::codec::{Framed, LinesCodec};

pub struct NetworkBridge {
    socket: Framed<TcpStream, LinesCodec>,
    rx: Receiver<String>,
}

impl NetworkBridge {
    pub fn new(
        socket: Framed<TcpStream, LinesCodec>,
        rx: Receiver<String>,
    ) -> NetworkBridge {
        NetworkBridge { socket, rx }
    }

    pub async fn listen(&mut self) -> () {
        loop {
            tokio::select! {
                // receive packet from the server
                frame = self.socket.next() => {
                    match frame {
                        Some(Ok(line)) => {
                            println!("[on_network] new line: {}", line);
                            // parse line
                            // emit via receiver
                        },
                        Some(Err(e)) => {
                            eprintln!("[on_network] error reading from socket: {}", e);
                            break;
                        },
                        None => {
                            println!("[on_network] socket closed");
                            break;
                        }
                    }
                },
                // send packet to the server
                packet = self.rx.recv() => {
                    match packet {
                        Some(payload) => {
                            if let Err(e) = self.socket.send(payload).await {
                                eprintln!("[on_network] error sending to socket: {}", e);
                            }
                        },
                        None => {}
                    }
                }
            }
        }

        println!("[on_network] connection closed");
    }
}
