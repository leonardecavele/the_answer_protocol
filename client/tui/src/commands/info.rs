use super::{Command, send_result};
use crate::events::AppEvent;
use api_client::client::Client;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct LookCommand;

#[async_trait]
impl Command for LookCommand {
    fn name(&self) -> &'static str {
        "look"
    }

    async fn execute(
        &self,
        _args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        let mut c = client.lock().await;
        match c.look().await {
            Ok(Ok(data)) => {
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::UpdateRoomContext {
                    room_id: data.room.id.clone(),
                    room_display_name: data.room.name.clone(),
                    npcs: data.npcs.clone(),
                }));
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::CommandResult(format!("{:#?}", data))));
            }
            Ok(Err(e)) => {
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::CommandError(e)));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Network(crate::events::NetEvent::TapError(e)));
            }
        }
    }
}

pub struct WhoCommand;

#[async_trait]
impl Command for WhoCommand {
    fn name(&self) -> &'static str {
        "who"
    }

    async fn execute(
        &self,
        _args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        let c = client.lock().await;
        match c.who().await {
            Ok(Ok(data)) => {
                let count = data.player_count as u32;
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::UpdateOnlinePlayers(count)));
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::CommandResult(format!("{:#?}", data))));
            }
            Ok(Err(e)) => {
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::CommandError(e)));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Network(crate::events::NetEvent::TapError(e)));
            }
        }
    }
}

pub struct InventoryCommand;

#[async_trait]
impl Command for InventoryCommand {
    fn name(&self) -> &'static str {
        "inventory"
    }

    async fn execute(
        &self,
        _args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        let c = client.lock().await;
        match c.inventory().await {
            Ok(Ok(data)) => {
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::InventoryUpdate(data.inventory.clone())));
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::CommandResult(format!("{:#?}", data))));
            }
            Ok(Err(e)) => {
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::CommandError(e)));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Network(crate::events::NetEvent::TapError(e)));
            }
        }
    }
}

pub struct StatusCommand;

#[async_trait]
impl Command for StatusCommand {
    fn name(&self) -> &'static str {
        "status"
    }

    async fn execute(
        &self,
        _args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        let c = client.lock().await;
        match c.status().await {
            Ok(Ok(data)) => {
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::UpdateStatus {
                    hp: data.player_status.hp,
                    max_hp: data.player_status.max_hp,
                }));
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::CommandResult(format!("{:#?}", data))));
            }
            Ok(Err(e)) => {
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::CommandError(e)));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Network(crate::events::NetEvent::TapError(e)));
            }
        }
    }
}

pub struct QuestsCommand;

#[async_trait]
impl Command for QuestsCommand {
    fn name(&self) -> &'static str {
        "quests"
    }

    async fn execute(
        &self,
        _args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        let c = client.lock().await;
        send_result(c.quests().await, &tx);
    }
}
