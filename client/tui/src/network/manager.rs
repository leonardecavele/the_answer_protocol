use crate::events::{ApplicationEvent, NetworkEvent};
use crate::network::commands::NetworkCommand;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// The NetworkManager is responsible for handling the connection to the game server.
/// It runs in a background Tokio task to ensure the UI never freezes during I/O.
pub struct NetworkManager {
    background_task: JoinHandle<()>,
    pub command_sender: mpsc::Sender<NetworkCommand>,
}

impl NetworkManager {
    /// Spawns the background network task.
    /// Takes a clone of the event broker sender to push network events to the main loop.
    pub fn start(
        event_sender: mpsc::Sender<ApplicationEvent>,
        server_ip: String,
        server_port: String,
        player_name: String,
    ) -> Self {
        let (command_tx, mut command_rx) = mpsc::channel::<NetworkCommand>(128);

        let background_task = tokio::spawn(async move {
            let server_address = format!("{}:{}", server_ip, server_port);

            match api_client::client::connect::ClientConnect::connect(&server_address).await {
                Ok(mut client) => {
                    // TCP Handshake OK. Step 2: Logical authentication
                    match client.connect(player_name.clone()).await {
                        Ok(Ok(_connect_response)) => {
                            let _ = event_sender
                                .send(ApplicationEvent::Network(NetworkEvent::ConnectionEstablished {
                                    server_ip,
                                    server_port,
                                    player_name,
                                }))
                                .await;

                            client.on_event({
                                let _event_sender = event_sender.clone();
                                move |server_event| {
                                    tracing::debug!("Received event from server: {:?}", server_event);
                                    let _ = _event_sender.try_send(ApplicationEvent::Network(
                                        NetworkEvent::ServerPayloadReceived(server_event),
                                    ));
                                }
                            });

                            // Command loop
                            while let Some(cmd) = command_rx.recv().await {
                                match cmd {
                                    NetworkCommand::Look => { let _ = client.look().await; }
                                    NetworkCommand::Move(dir) => { let _ = client.r#move(dir).await; }
                                    NetworkCommand::ChatGlobal(msg) => { let _ = client.chat_global(msg).await; }
                                    NetworkCommand::ChatPrivate { to, message } => { let _ = client.chat_private(to, message).await; }
                                    NetworkCommand::Who => { let _ = client.who().await; }
                                    NetworkCommand::GroupCreate => { let _ = client.group_create().await; }
                                    NetworkCommand::GroupInvite(u) => { let _ = client.group_invite(u).await; }
                                    NetworkCommand::GroupJoin(leader) => { let _ = client.group_join(leader).await; }
                                    NetworkCommand::GroupLeave => { let _ = client.group_leave().await; }
                                    NetworkCommand::Take(item) => { let _ = client.take(item).await; }
                                    NetworkCommand::DropItem(item) => { let _ = client.drop_item(item).await; }
                                    NetworkCommand::Inventory => { let _ = client.inventory().await; }
                                    NetworkCommand::Talk(npc) => { let _ = client.talk(npc).await; }
                                    NetworkCommand::Attack(npc) => { let _ = client.attack(npc).await; }
                                    NetworkCommand::Status => { let _ = client.status().await; }
                                    NetworkCommand::Quest(npc) => { let _ = client.quest(npc).await; }
                                    NetworkCommand::Quests => { let _ = client.quests().await; }
                                    NetworkCommand::Quit => {
                                        client.quit().await;
                                        break;
                                    }
                                }
                            }
                        }
                        Ok(Err(command_error)) => {
                            let _ = event_sender
                                .send(ApplicationEvent::Network(NetworkEvent::ConnectionFailed {
                                    error_message: format!("Login rejected: {:?}", command_error),
                                }))
                                .await;
                        }
                        Err(tap_error) => {
                            let _ = event_sender
                                .send(ApplicationEvent::Network(NetworkEvent::ConnectionFailed {
                                    error_message: format!("Communication error: {:?}", tap_error),
                                }))
                                .await;
                        }
                    }
                }
                Err(e) => {
                    let _ = event_sender
                        .send(ApplicationEvent::Network(NetworkEvent::ConnectionFailed {
                            error_message: format!("TCP error: {}", e),
                        }))
                        .await;
                }
            }
        });

        Self {
            background_task,
            command_sender: command_tx,
        }
    }

    pub fn send_command(&self, cmd: NetworkCommand) {
        let _ = self.command_sender.try_send(cmd);
    }
}

impl Drop for NetworkManager {
    fn drop(&mut self) {
        self.background_task.abort();
    }
}
