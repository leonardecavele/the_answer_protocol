use api_client::client::APIClient;
use std::env;
use std::process::exit;
use std::time::Instant;
use time::macros::format_description;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::LocalTime;

pub enum Command {
    Quit,
}

// const SERVER_ADDRESS: &str = "127.0.0.1:3000";
const SERVER_ADDRESS: &str = "127.0.0.1:38800";

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
    let player = env::var("PLAYER")
        .ok()
        .unwrap_or_else(|| PLAYER.to_string());
    match suffix {
        Some(v) => String::from(format!("{}_{}", player, v)),
        None => player,
    }
}

#[allow(dead_code)]
async fn test_multiple_connections() {
    let mut clients: Vec<APIClient> = vec![];

    for i in 0..3 {
        let player = get_player_name(Some(i.to_string()));

        let mut client = match APIClient::new(SERVER_ADDRESS).await {
            Ok(client) => client,
            Err(e) => {
                eprintln!(
                    "couldn't connect to the server ({}): {}. Exit.",
                    SERVER_ADDRESS, e
                );
                return;
            }
        };

        let player_name = player.clone();
        client
            .on_event(move |event| info!("[new event for {}]: {:?}", player_name, event.arguments));

        if !player_connect(&mut client, player).await {
            exit(1);
        }
        if !player_look(&mut client).await {
            exit(1);
        }

        clients.push(client);
    }

    for client in clients {
        client.quit().await;
    }
}

#[allow(dead_code)]
async fn test_single_connection() {
    let player = get_player_name(None);

    let mut client = match APIClient::new(SERVER_ADDRESS).await {
        Ok(client) => client,
        Err(e) => {
            error!(
                "couldn't connect to the server ({}): {}. Exit.",
                SERVER_ADDRESS, e
            );
            return;
        }
    };

    let player_name = player.clone();
    client.on_event(move |event| info!("[new event for {}]: {:?}", player_name, event.arguments));

    if !player_connect(&mut client, player).await {
        exit(1);
    }

    let iterations = 10000;
    info!(
        "Démarrage du benchmark pour {} requêtes LOOK...",
        iterations
    );
    let start_time = Instant::now();

    for i in 0..10000 {
        if !player_look(&mut client).await {
            exit(1);
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

async fn player_connect(client: &mut APIClient, player: String) -> bool {
    match client.connect(player).await {
        Ok(Ok(response)) => {
            println!("connected to the server as {}.", response.player_name);
            true
        }
        Ok(Err(e)) => {
            error!("{}", e);
            true
        }
        Err(e) => {
            error!("fail to connect player: {}", e);
            false
        }
    }
}

async fn player_look(client: &mut APIClient) -> bool {
    match client.look().await {
        Ok(Ok(response)) => {
            println!("look response: {}.", response.json_data);
            true
        }
        Ok(Err(e)) => {
            error!("{}", e);
            true
        }
        Err(e) => {
            eprintln!("fail to look: {}", e);
            false
        }
    }
}
