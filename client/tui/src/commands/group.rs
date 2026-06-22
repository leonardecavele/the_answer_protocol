use super::{Command, send_result};
use crate::events::AppEvent;
use api_client::client::Client;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct GroupCreateCommand;

#[async_trait]
impl Command for GroupCreateCommand {
    fn name(&self) -> &'static str {
        "group_create"
    }

    async fn execute(
        &self,
        _args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        let mut c = client.lock().await;
        match c.group_create().await {
            Ok(Ok(data)) => {
                let _ = tx.send(AppEvent::UpdateGroup(Some(data.group_id.clone())));
                let _ = tx.send(AppEvent::CommandResult(format!("{:#?}", data)));
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

pub struct GroupInviteCommand;

#[async_trait]
impl Command for GroupInviteCommand {
    fn name(&self) -> &'static str {
        "group_invite"
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
        send_result(c.group_invite(args[0].clone()).await, &tx);
    }
}

pub struct GroupJoinCommand;

#[async_trait]
impl Command for GroupJoinCommand {
    fn name(&self) -> &'static str {
        "group_join"
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
        match c.group_join(args[0].clone()).await {
            Ok(Ok(data)) => {
                let _ = tx.send(AppEvent::UpdateGroup(Some(data.group_id.clone())));
                let _ = tx.send(AppEvent::CommandResult(format!("{:#?}", data)));
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

pub struct GroupLeaveCommand;

#[async_trait]
impl Command for GroupLeaveCommand {
    fn name(&self) -> &'static str {
        "group_leave"
    }

    async fn execute(
        &self,
        _args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    ) {
        let mut c = client.lock().await;
        match c.group_leave().await {
            Ok(Ok(data)) => {
                let _ = tx.send(AppEvent::UpdateGroup(None));
                let _ = tx.send(AppEvent::CommandResult(format!("{:#?}", data)));
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
