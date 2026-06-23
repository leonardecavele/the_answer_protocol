use crate::events::{AppEvent, GameEvent, NetEvent};
use api_client::client::Client;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod action;
pub mod chat;
pub mod group;
pub mod info;

pub fn send_result<T: std::fmt::Debug>(
    res: Result<Result<T, api_client::error::CommandError>, api_client::error::TapError>,
    tx: &tokio::sync::mpsc::Sender<AppEvent>,
) {
    match res {
        Ok(Ok(data)) => {
            let _ = tx.send(AppEvent::Game(GameEvent::CommandResult(format!("{:#?}", data))));
        }
        Ok(Err(e)) => {
            let _ = tx.send(AppEvent::Game(GameEvent::CommandError(e)));
        }
        Err(e) => {
            let _ = tx.send(AppEvent::Network(NetEvent::TapError(e)));
        }
    }
}

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &'static str;
    async fn execute(
        &self,
        args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    );
}

pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        registry.register_all();
        registry
    }

    fn register(&mut self, cmd: Box<dyn Command>) {
        self.commands.insert(cmd.name().to_string(), cmd);
    }

    fn register_all(&mut self) {
        // Chat
        self.register(Box::new(chat::ChatGlobalCommand));
        self.register(Box::new(chat::ChatPrivateCommand));
        self.register(Box::new(chat::TalkCommand));

        // Info
        self.register(Box::new(info::LookCommand));
        self.register(Box::new(info::WhoCommand));
        self.register(Box::new(info::InventoryCommand));
        self.register(Box::new(info::StatusCommand));
        self.register(Box::new(info::QuestsCommand));

        // Action
        self.register(Box::new(action::TakeCommand));
        self.register(Box::new(action::DropCommand));
        self.register(Box::new(action::MoveCommand));
        self.register(Box::new(action::AttackCommand));
        self.register(Box::new(action::QuestCommand));

        // Group
        self.register(Box::new(group::GroupCreateCommand));
        self.register(Box::new(group::GroupInviteCommand));
        self.register(Box::new(group::GroupJoinCommand));
        self.register(Box::new(group::GroupLeaveCommand));
    }

    pub async fn execute(
        &self,
        cmd_name: &str,
        args: Vec<String>,
        client: Arc<Mutex<Client>>,
        tx: tokio::sync::mpsc::Sender<AppEvent>,
    ) -> bool {
        if let Some(cmd) = self.commands.get(cmd_name) {
            cmd.execute(args, client, tx).await;
            true
        } else {
            false
        }
    }
}

pub fn handle_command(
    state: &mut crate::state::AppState,
    cmd_line: String,
    tx: tokio::sync::mpsc::Sender<AppEvent>,
) {
    let registry = std::sync::Arc::clone(&state.registry);
    let parts: Vec<&str> = cmd_line.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd = parts[0].to_string();
    state.game.push_game_output(format!("> {}", cmd_line));

    if cmd == "quit" {
        state.should_quit = true;
        return;
    }

    if let Some(client_arc) = &state.net.client {
        let client_arc = Arc::clone(client_arc);
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let handled = registry.execute(&cmd, args, client_arc, tx_clone.clone()).await;
            if !handled {
                let _ = tx_clone.send(AppEvent::Game(GameEvent::UnknowCommand(cmd)));
            }
        });
    } else {
        state.game.push_game_output("Not connected.".to_string());
    }
}
