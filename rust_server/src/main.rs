use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};

fn start_reader_thread(reader_stream: TcpStream, shared_buffer: Arc<Mutex<Vec<String>>>)
{
    thread::spawn(move || {
        let reader = BufReader::new(reader_stream);

        for line in reader.lines() {
            match line {
                Ok(message) => {
                    let mut buffer = shared_buffer.lock().unwrap();
                    buffer.push(message);
                }
                Err(err) => {
                    eprintln!("Read error: {}", err);
                    break;
                }
            }
        }
    });
}

fn need_to_print(buffer: &Vec<String>) -> bool 
{

    if buffer.len() == 0 {
        false
    }
    else if buffer[0].as_str() == "PING" {
        true
    }
    else{
        false
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:38801")?;

    println!("Rust server started on 38801");

    let (mut writer_stream, addr) = listener.accept()?;

    println!("Go connected: {}", addr);

    let reader_stream = writer_stream.try_clone()?;

    let shared_buffer = Arc::new(Mutex::new(Vec::<String>::new()));
    start_reader_thread(reader_stream, Arc::clone(&shared_buffer));
    loop {
        let mut buffer = shared_buffer.lock().unwrap();
        let print_pong = need_to_print(&buffer) ;

        if print_pong {
            buffer.clear();
            writer_stream.write_all(b"PONG\n")?;
        }
    }
}