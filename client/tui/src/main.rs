use api_client::client::Client;
use api_client::client::connect::ClientConnect;
use api_client::client::dispatcher::ServerEvent;
use api_client::error::TapError;
use std::io::{Write, stdin};
use std::process::exit;
use std::time::Instant;
use std::{env, io};
use time::macros::format_description;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::LocalTime;

pub enum Command {
    Quit,
}

const LOCAL_SERVER_IP: &str = "127.0.0.1";
const LOCAL_SERVER_PORT: &str = "38800";
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

fn get_server_ip() -> String {
    env::var("SERVER_IP")
        .ok()
        .unwrap_or_else(|| LOCAL_SERVER_IP.to_string())
}

fn get_server_port() -> String {
    env::var("SERVER_PORT")
        .ok()
        .unwrap_or_else(|| LOCAL_SERVER_PORT.to_string())
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

        let mut client =
            ClientConnect::connect(format!("{}:{}", get_server_ip(), get_server_port())).await?;

        client.on_event(move |event| match event {
            ServerEvent::Connect(name) => {
                info!("[connect event]: new player {:?}", &name);
            }
            ServerEvent::Quit(name) => {
                info!("[quit event]: player {:?} as quit", &name);
            }
            ServerEvent::GlobalChat(data) => {
                info!("[global event]: {:?}", data);
            }
            ServerEvent::PrivateChat(data) => {
                info!("[private event]: {:?}", data);
            }
            ServerEvent::RoomPresence(data) => {
                info!("[room presence event]: {:?}", data);
            }
            ServerEvent::Unknown(data) => {
                error!("[unknown event]: {:?}", data);
            }
        });

        player_connect(&mut client, player).await?;
        player_look(&mut client).await?;

        clients.push(client);
    }

    loop {
        clients.retain(|client| !client.is_connected());

        if clients.is_empty() {
            break;
        }
    }

    Ok(())
}

#[allow(dead_code)]
async fn test_single_connection() -> Result<(), TapError> {
    let mut client =
        ClientConnect::connect(format!("{}:{}", get_server_ip(), get_server_port())).await?;

    // player_connect(&mut client, player.clone()).await?;
    // player_chat_global(&mut client, player.clone()).await?;
    //
    // let iterations = 10000;
    // info!(
    //     "Démarrage du benchmark pour {} requêtes LOOK...",
    //     iterations
    // );
    // let start_time = Instant::now();
    //
    // for i in 0..1 {
    //     player_look(&mut client).await?;
    //     info!("loop {}", i);
    // }
    //
    // let duration = start_time.elapsed();
    //
    // info!(
    //     "Benchmark terminé : {} requêtes en {:.2?}. (Moyenne : {:.2?} / requête)",
    //     iterations,
    //     duration,
    //     duration / iterations
    // );

    client.on_event(move |event| match event {
        ServerEvent::Connect(name) => {
            info!("[connect event]: new player {:?}", &name);
        }
        ServerEvent::Quit(name) => {
            info!("[quit event]: player {:?} as quit", &name);
        }
        ServerEvent::GlobalChat(data) => {
            info!("[global event]: {:?}", data);
        }
        ServerEvent::PrivateChat(data) => {
            info!("[private event]: {:?}", data);
        }
        ServerEvent::RoomPresence(data) => {
            info!("[room presence event]: {:?}", data);
        }
        ServerEvent::Unknown(data) => {
            error!("[unknown event]: {:?}", data);
        }
    });

    let mut stdin = stdin();
    let input = &mut String::new();

    loop {
        if !client.is_connected() {
            break;
        }

        input.clear();
        print!("(connect | say_hello_global | look)> ");
        let _ = io::stdout().flush();
        let _ = stdin.read_line(input);

        let _ = match input.trim().split(" ").collect::<Vec<&str>>().as_slice() {
            ["connect", name] => {
                let _ = player_connect(&mut client, name.to_string()).await;
            }
            ["say_hello_global", message @ ..] => {
                let _ = client.chat_global(message.join(" ")).await?;
            }
            ["look"] => {
                let _ = player_look(&mut client).await;
            }
            ["quit"] => {
                client.quit().await;
                break;
            }
            _ => {}
        };
    }

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

async fn player_chat_global(client: &mut Client, player: String) -> Result<(), TapError> {
    let result = client.chat_global(format!("Hello from {}", player)).await?;

    match result {
        Ok(response) => {}
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
            info!("look response: {:?}.", response);
        }
        Err(e) => {
            error!("{}", e);
        }
    }

    Ok(())
}
