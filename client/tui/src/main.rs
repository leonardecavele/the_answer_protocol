use api_client::client::APIClient;
use std::process::exit;

pub enum Command {
    Quit,
}

// const SERVER_ADDRESS: &str = "127.0.0.1:3000";
const SERVER_ADDRESS: &str = "10.14.4.3:38800";

const PLAYER: &str = "Alice";

#[tokio::main]
async fn main() {
    // let (tx, mut rx) = mpsc::channel::<Command>(1024);

    let client = match APIClient::new(SERVER_ADDRESS).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!(
                "Couldn't connect to the server ({}): {}. Exit.",
                SERVER_ADDRESS, e
            );
            return;
        }
    };

    println!(
        "Connection established to {} (Ver. {})",
        client.server.addr, client.server.protocol_version
    );

    if let Err(e) = client.connect(PLAYER.to_string()).await {
        eprintln!("Fail to connect player: {}", e);
        exit(1);
    }

    println!("Connected to the server as {}", PLAYER);

    // client
    //     .on_event(|message| {
    //         println!("New Event {:?}", message);
    //     })
    //     .await;

    loop {}
}
