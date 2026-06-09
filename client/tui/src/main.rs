use api_client::client::APIClient;
use std::process::exit;
use time::macros::format_description;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::LocalTime;

pub enum Command {
    Quit,
}

const SERVER_ADDRESS: &str = "127.0.0.1:3000";
// const SERVER_ADDRESS: &str = "10.14.4.3:38800";

const PLAYER: &str = "Alice";

#[tokio::main]
async fn main() {
    let time_format = format_description!("[hour]:[minute]:[second]");
    let timer = LocalTime::new(time_format);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_timer(timer)
        .init();

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

    if let Err(e) = client.connect(PLAYER.to_string()).await {
        eprintln!("Fail to connect player: {}", e);
        client.exit();
        exit(1);
    }

    // client
    //     .on_event(|message| {
    //         println!("New Event {:?}", message);
    //     })
    //     .await;

    loop {}
}
