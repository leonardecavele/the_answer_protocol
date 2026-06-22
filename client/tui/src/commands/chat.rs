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
        tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        let mut c = client.lock().await;
        let msg = args.join(" ");
        match c.chat_global(msg.clone()).await {
            Ok(Ok(_res)) => {
                let _ = tx.send(AppEvent::LocalChatSent(ChatScope::Global, msg));
            }
            Ok(Err(e)) => {
                let _ = tx.send(AppEvent::CommandError(e));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::TapError(e));
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
        tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        if args.len() < 2 {
            return;
        }
        let to = args[0].clone();
        let msg = args[1..].join(" ");
        let mut c = client.lock().await;
        match c.chat_private(to.clone(), msg.clone()).await {
            Ok(Ok(_res)) => {
                let _ = tx.send(AppEvent::LocalChatSent(
                    ChatScope::Private,
                    format!("to {}: {}", to, msg),
                ));
            }
            Ok(Err(e)) => {
                let _ = tx.send(AppEvent::CommandError(e));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::TapError(e));
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
        tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        if args.is_empty() {
            return;
        }
        let mut c = client.lock().await;
        send_result(c.talk(args.join(" ")).await, &tx);
    }
}
