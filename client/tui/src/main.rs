use api_client::APIClient;

pub enum Command {
    Quit,
}

const SERVER_ADDRESS: &str = "127.0.0.1:3000";

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

    // client
    //     .on_event(|message| {
    //         println!("New Event {:?}", message);
    //     })
    //     .await;

    loop {
        // match rx.recv().await {
        //     Some(Command::Quit) => break,
        //     None => {}
        // }
    }
}
