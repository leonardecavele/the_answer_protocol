use super::{Command, send_result};
use crate::events::AppEvent;
use crate::state::ChatScope;
use api_client::client::Client;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ChatGlobalCommand;

#[async_trait]
impl Command for ChatGlobalCommand {
    fn name(&self) -> &'static str {
        "chat_global"
    }

    async fn execute(
        &self,
        args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        let c = client.lock().await;
        match c.chat_global(args.join(" ")).await {
            Ok(Ok(_)) => {
                let msg = args.join(" ");
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::LocalChatSent(ChatScope::Global, msg)));
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

pub struct ChatPrivateCommand;

#[async_trait]
impl Command for ChatPrivateCommand {
    fn name(&self) -> &'static str {
        "chat_private"
    }

    async fn execute(
        &self,
        args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) {
        if args.len() < 2 {
            return;
        }
        let to = args[0].clone();
        let msg = args[1..].join(" ");
        let c = client.lock().await;
        match c.chat_private(to, msg.clone()).await {
            Ok(Ok(_)) => {
                let _ = tx.send(AppEvent::Game(crate::events::GameEvent::LocalChatSent(ChatScope::Private, msg)));
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

pub struct TalkCommand;

#[async_trait]
impl Command for TalkCommand {
    fn name(&self) -> &'static str {
        "talk"
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
        send_result(c.talk(args.join(" ")).await, &tx);
    }
}
