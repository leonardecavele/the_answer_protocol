use api_client::client::connect::ClientConnect;
use api_client::client::event::{GroupEvent, RoomEvent, ServerEvent};
use api_client::error::TapError;
use std::io::{stdin, Write};
use std::{env, io};
use time::macros::format_description;
use tracing::{error, info};
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::EnvFilter;

const LOCAL_SERVER_IP: &str = "127.0.0.1";
const LOCAL_SERVER_PORT: &str = "38800";

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

#[allow(dead_code)]
async fn test_single_connection() -> Result<(), TapError> {
    let mut client =
        ClientConnect::connect(format!("{}:{}", get_server_ip(), get_server_port())).await?;

    client.on_connect_event(|name| {
        info!("[connect event]: new player {:?}", name);
    });

    client.on_quit_event(|name| {
        info!("[quit event]: player {:?} has quit", name);
    });

    client.on_global_chat_event(|msg| {
        info!("[global chat]: {}: {}", msg.sender, msg.message);
    });

    client.on_private_chat_event(|msg| {
        info!("[private chat]: {}: {}", msg.sender, msg.message);
    });

    client.on_room_event(|event| match event {
        RoomEvent::PresenceEnter(name) => {
            info!("[room presence]: {:?} entered the room", name);
        }
        RoomEvent::PresenceLeave(name) => {
            info!("[room presence]: {:?} left the room", name);
        }
        RoomEvent::Chat(msg) => {
            info!("[room chat]: {}: {}", msg.sender, msg.message);
        }
    });

    client.on_group_event(|event| match event {
        GroupEvent::Invite(leader) => {
            info!("[group invite]: invited by {:?}", leader);
        }
        GroupEvent::Join(user) => {
            info!("[group join]: player {:?} joined the group", user);
        }
        GroupEvent::Leave(user) => {
            info!("[group leave]: player {:?} left the group", user);
        }
        GroupEvent::Chat(msg) => {
            info!("[group chat]: {}: {}", msg.sender, msg.message);
        }
    });

    client.on_stats_event(|count| {
        info!("[stats event]: players online: {}", count);
    });

    client.on_event(|event| {
        if let ServerEvent::Unknown(data) = event {
            error!("[unknown event]: {:?}", data);
        }
    });

    let stdin = stdin();
    let input = &mut String::new();

    let mut player_name = String::new();

    loop {
        if !client.is_connected() {
            break;
        }

        input.clear();
        println!(
            "(connect | chat_global | chat_private | look | who | group_create | group_invite | group_join | group_leave | take | drop | inventory | talk | attack | status | quest | quests | quit)"
        );
        print!("> ");
        let _ = io::stdout().flush();
        let _ = stdin.read_line(input);

        let _ = match input.trim().split(" ").collect::<Vec<&str>>().as_slice() {
            ["connect", name] => {
                let result = client.connect(name.to_string()).await?;
                match result {
                    Ok(response) => {
                        player_name = response.player_name;
                    },
                    Err(error) => {
                        error!("[connect error]: {}", error);
                    }
                };
                println!("[connect] {:?}", player_name);
            }
            ["chat_global", message @ ..] => {
                let result = client.chat_global(message.join(" ")).await;
                println!("[chat_global] {:?}", result);
            }
            ["chat_private", to, message @ ..] => {
                let result = client.chat_private(to.to_string(), message.join(" ")).await;
                println!("[chat_private] {:?}", result);
            }
            ["look"] => {
                let result = client.look().await;
                println!("[look {}] {:?}", player_name, result);
            }
            ["who"] => {
                let result = client.who().await;
                println!("[who] {:?}", result);
            }
            ["group_create"] => {
                let result = client.group_create().await;
                println!("[group_create] {:?}", result);
            }
            ["group_invite", name] => {
                let result = client.group_invite(name.to_string()).await;
                println!("[group_invite] {:?}", result);
            }
            ["group_join", leader_name] => {
                let result = client.group_join(leader_name.to_string()).await;
                println!("[group_join] {:?}", result);
            }
            ["group_leave"] => {
                let result = client.group_leave().await;
                println!("[group_leave] {:?}", result);
            }
            ["take", item] => {
                let result = client.take(item.to_string()).await;
                println!("[take] {:?}", result);
            }
            ["drop", item] => {
                let result = client.drop_item(item.to_string()).await;
                println!("[drop] {:?}", result);
            }
            ["inventory"] => {
                let result = client.inventory().await;
                println!("[inventory] {:?}", result);
            }
            ["talk", npc] => {
                let result = client.talk(npc.to_string()).await;
                println!("[talk] {:?}", result);
            }
            ["attack", npc] => {
                let result = client.attack(npc.to_string()).await;
                println!("[attack] {:?}", result);
            }
            ["status"] => {
                let result = client.status().await;
                println!("[status] {:?}", result);
            }
            ["quest", npc] => {
                let result = client.quest(npc.to_string()).await;
                println!("[quest] {:?}", result);
            }
            ["quests"] => {
                let result = client.quests().await;
                println!("[quests] {:?}", result);
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
