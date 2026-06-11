use api_client::client::Client;
use api_client::client::connect::ClientConnect;
use api_client::error::TapError;
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

const SERVER_ADDRESS: &str = "127.0.0.1:3000";
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

    if let Err(e) = test_single_connection().await {
        error!("{} (exiting)", e);
    }
    // if let Err(e) = test_multiple_connections().await {
    //     error!("{} (exiting)", e);
    // }
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
async fn test_multiple_connections() -> Result<(), TapError> {
    let mut clients: Vec<Client> = vec![];

    for i in 0..3 {
        let player = get_player_name(Some(i.to_string()));

        let mut client = ClientConnect::connect(SERVER_ADDRESS).await?;

        let player_name = player.clone();
        client
            .on_event(move |event| info!("[new event for {}]: {:?}", player_name, event.arguments));

        player_connect(&mut client, player).await?;
        player_look(&mut client).await?;

        clients.push(client);
    }

    for client in clients {
        client.quit().await;
    }

    Ok(())
}

#[allow(dead_code)]
async fn test_single_connection() -> Result<(), TapError> {
    let player = get_player_name(None);

    let mut client = ClientConnect::connect(SERVER_ADDRESS).await?;

    let player_name = player.clone();
    client.on_event(move |event| info!("[new event for {}]: {:?}", player_name, event.arguments));

    player_connect(&mut client, player).await?;

    let iterations = 10000;
    info!(
        "Démarrage du benchmark pour {} requêtes LOOK...",
        iterations
    );
    let start_time = Instant::now();

    for i in 0..200 {
        player_look(&mut client).await?;
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

    Ok(())
}

async fn player_connect(client: &mut Client, player: String) -> Result<(), TapError> {
    let result = client.connect(player).await?;

    match result {
        Ok(response) => {
            println!("connected to the server as {}.", response.player_name);
        }
        Err(e) => {
            error!("{}", e);
        }
    }

    Ok(())
}

async fn player_look(client: &mut Client) -> Result<(), TapError> {
    let result = client.look().await?;

    match result {
        Ok(response) => {
            println!("look response: {}.", response.json_data);
        }
        Err(e) => {
            error!("{}", e);
        }
    }

    Ok(())
}
