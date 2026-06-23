use super::{Command, send_result};
use crate::events::AppEvent;
use api_client::client::Client;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TakeCommand;

#[async_trait]
impl Command for TakeCommand {
    fn name(&self) -> &'static str {
        "take"
    }

    async fn execute(
        &self,
        args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        if args.is_empty() {
            return;
        }
        let c = client.lock().await;
        send_result(c.take(args[0].clone()).await, &tx);
    }
}

pub struct DropCommand;

#[async_trait]
impl Command for DropCommand {
    fn name(&self) -> &'static str {
        "drop"
    }

    async fn execute(
        &self,
        args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        if args.is_empty() {
            return;
        }
        let c = client.lock().await;
        send_result(c.drop_item(args[0].clone()).await, &tx);
    }
}

pub struct MoveCommand;

#[async_trait]
impl Command for MoveCommand {
    fn name(&self) -> &'static str {
        "move"
    }

    async fn execute(
        &self,
        args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        if args.is_empty() {
            return;
        }
        let mut c = client.lock().await;
        match c.r#move(args[0].clone().to_uppercase()).await {
            Ok(Ok(data)) => {
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::CommandResult(format!("{:#?}", data))));
                
                // Fetch the new room state immediately
                match c.look().await {
                    Ok(Ok(look_data)) => {
                        let _ = tx.send(AppEvent::Game(crate::events::GameEvent::UpdateRoomContext {
                            room_id: look_data.room.id.clone(),
                            room_display_name: look_data.room.name.clone(),
                            npcs: look_data.npcs.clone(),
                        }));
                        let _ = tx.send(AppEvent::Game(crate::events::GameEvent::CommandResult(format!("{:#?}", look_data))));
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(AppEvent::Game(crate::events::GameEvent::CommandError(e)));
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::Network(crate::events::NetEvent::TapError(e)));
                    }
                }
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

pub struct AttackCommand;

#[async_trait]
impl Command for AttackCommand {
    fn name(&self) -> &'static str {
        "attack"
    }

    async fn execute(
        &self,
        args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        if args.is_empty() {
            return;
        }
        let c = client.lock().await;
        send_result(c.attack(args.join(" ")).await, &tx);
    }
}

pub struct QuestCommand;

#[async_trait]
impl Command for QuestCommand {
    fn name(&self) -> &'static str {
        "quest"
    }

    async fn execute(
        &self,
        args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        if args.is_empty() {
            return;
        }
        let c = client.lock().await;
        send_result(c.quest(args.join(" ")).await, &tx);
    }
}
