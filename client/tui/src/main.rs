use api_client::client::APIClient;
use api_client::protocol::command::CommandResult;
use std::process::exit;
use time::macros::format_description;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::LocalTime;

pub enum Command {
    Quit,
}

// const SERVER_ADDRESS: &str = "127.0.0.1:3000";
const SERVER_ADDRESS: &str = "10.12.10.5:38800";

const PLAYER: &str = "Player";

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

    test_single_connection().await;
    // test_multiple_connections().await;

    // client
    //     .on_event(|message| {
    //         println!("New Event {:?}", message);
    //     })
    //     .await;

    loop {}
}

#[allow(dead_code)]
async fn test_multiple_connections() {
    let mut clients: Vec<APIClient> = vec![];

    for i in 0..3 {
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

        let player = format!("{}_{}", PLAYER, i);

        match client.connect(player).await {
            Ok(result) => match result {
                CommandResult::Success { data } => {
                    println!("Connected to the server as {}.", data.player_name);
                }
                CommandResult::Error { message } => {
                    println!("[tui] {}", message);
                }
            },
            Err(e) => {
                eprintln!("Fail to connect player: {}", e);
                client.close();
                exit(1);
            }
        }

        match client.look().await {
            Ok(result) => match result {
                CommandResult::Success { data } => {
                    println!("Look response: {}", data.json_data);
                }
                CommandResult::Error { message } => {
                    println!("[tui] {}", message);
                }
            },
            Err(e) => {
                eprintln!("Fail to look: {}", e);
                client.close();
                exit(1);
            }
        }

        clients.push(client);
    }

    for client in clients {
        client.close()
    }
}

#[allow(dead_code)]
async fn test_single_connection() {
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

    let player = PLAYER.to_string();

    match client.connect(player).await {
        Ok(result) => match result {
            CommandResult::Success { data } => {
                println!("Connected to the server as {}.", data.player_name);
            }
            CommandResult::Error { message } => {
                println!("[tui] {}", message);
            }
        },
        Err(e) => {
            eprintln!("Fail to connect player: {}", e);
            client.close();
            exit(1);
        }
    }

    match client.look().await {
        Ok(result) => match result {
            CommandResult::Success { data } => {
                println!("Look response: {}", data.json_data);
            }
            CommandResult::Error { message } => {
                println!("[tui] {}", message);
            }
        },
        Err(e) => {
            eprintln!("Fail to look: {}", e);
            client.close();
            exit(1);
        }
    }
}
