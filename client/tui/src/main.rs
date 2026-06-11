use api_client::client::APIClient;
use api_client::protocol::command::CommandResult;
use std::collections::HashMap;
use std::env;
use std::process::exit;
use std::time::Instant;
use time::macros::format_description;
use tracing::{debug, info};
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::EnvFilter;

pub enum Command {
    Quit,
}

// const SERVER_ADDRESS: &str = "127.0.0.1:3000";
const SERVER_ADDRESS: &str = "10.14.8.5:38800";

const PLAYER: &str = "DefaultPlayer";

#[tokio::main]
async fn main() {
    let time_format = format_description!("[hour]:[minute]:[second].[subsecond digits:6]");
    let timer = LocalTime::new(time_format);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_timer(timer)
        .init();

    test_single_connection().await;
    // test_multiple_connections().await;
}

fn get_player_name(suffix: Option<String>) -> String {
    let player = env::var("PLAYER").ok().unwrap_or_else(|| PLAYER.to_string());
    match suffix {
        Some(v) => String::from(format!("{}_{}", player, v)),
        None => player,
    }
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

        let player = get_player_name(Some(i.to_string()));

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
    let mut client = match APIClient::new(SERVER_ADDRESS).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!(
                "Couldn't connect to the server ({}): {}. Exit.",
                SERVER_ADDRESS, e
            );
            return;
        }
    };

    client.on_event(|event| {
        info!("[new event] : {:?}", event.arguments)
    });

    let player = get_player_name(None);

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

    // client.quit().await;

    let iterations = 10000;
    info!("Démarrage du benchmark pour {} requêtes LOOK...", iterations);
    let start_time = Instant::now();

    for i in 0..10000 {
        match client.look().await {
            Ok(result) => match result {
                CommandResult::Success { data } => {
                    debug!("Look response: {}", data.json_data);
                }
                CommandResult::Error { message } => {
                    debug!("[tui] {}", message);
                }
            },
            Err(e) => {
                eprintln!("Fail to look: {}", e);
                client.close();
                exit(1);
            }
        }
        info!("loop {}", i);
    }

    let duration = start_time.elapsed();

    info!(
        "Benchmark terminé : {} requêtes en {:.2?}. (Moyenne : {:.2?} / requête)",
        iterations,
        duration,
        duration / iterations
    );

    client.quit().await;
}
