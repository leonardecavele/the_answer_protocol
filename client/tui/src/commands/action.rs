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
        tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        if args.is_empty() {
            return;
        }
        let mut c = client.lock().await;
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
        tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        if args.is_empty() {
            return;
        }
        let mut c = client.lock().await;
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
        tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        if args.is_empty() {
            return;
        }
        let mut c = client.lock().await;
        send_result(c.r#move(args[0].clone().to_uppercase()).await, &tx);
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
        tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        if args.is_empty() {
            return;
        }
        let mut c = client.lock().await;
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
        tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        if args.is_empty() {
            return;
        }
        let mut c = client.lock().await;
        send_result(c.quest(args.join(" ")).await, &tx);
    }
}
