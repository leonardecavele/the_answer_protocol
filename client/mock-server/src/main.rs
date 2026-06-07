use std::io::Result;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use thread::sleep;

const PROTOCOL_VERSION: &str = "1.0";

fn handle_client(stream: &mut TcpStream) -> Result<()> {
    println!("New connection: {}", stream.peer_addr().unwrap());

    stream.write(format!("OK hello proto={}\n", PROTOCOL_VERSION).as_bytes())?;
    sleep(Duration::from_secs(1));

    stream.write("EVT USER ENTER\n".as_bytes())?;
    sleep(Duration::from_secs(1));

    let mut i = 0;
    while i < 5 {
        stream.write(format!("OK ping {}\n", i + 1).as_bytes())?;
        i += 1;
        sleep(Duration::from_secs(1));
    }

    stream.write("EVT USER ENTER\n".as_bytes())?;
    sleep(Duration::from_secs(1));
    stream.write("EVT USER LEAVE\n".as_bytes())?;
    sleep(Duration::from_secs(1));
    stream.write("ERR Server 400\n".as_bytes())?;

    Ok(())
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000")?;
    println!("Listening on: {}", listener.local_addr()?);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    let mut stream = stream;
                    handle_client(&mut stream)
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}
