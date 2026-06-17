use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{debug, warn};

pub struct EventDispatcher {
    broadcast_sender: broadcast::Sender<ServerEvent>,
    subscriber_tasks: Vec<JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub enum ServerEvent {
    Connect(String),
    Quit(String),
    Chat(ChatEventData),
    RoomPresence(RoomPresenceData),
    Unknown(String),
}

#[derive(Debug, Clone)]
pub enum ChatScopeType {
    Global,
    Room,
    Private,
}

#[derive(Debug, Clone)]
pub struct ChatEventData {
    pub scope: ChatScopeType,
    pub sender: String,
    pub message: String,
}

// Data for a room presence event
#[derive(Debug, Clone)]
pub enum RoomPresenceAction {
    Enter,
    Leave,
}

#[derive(Debug, Clone)]
pub struct RoomPresenceData {
    pub action: RoomPresenceAction,
    pub name: String,
}

impl EventDispatcher {
    pub fn new(broadcast_sender: broadcast::Sender<ServerEvent>) -> Self {
        Self {
            broadcast_sender,
            subscriber_tasks: vec![],
        }
    }

    pub fn subscribe<F>(&mut self, mut handler: F)
    where
        F: FnMut(ServerEvent) + Send + 'static,
    {
        let mut subscriber = self.broadcast_sender.subscribe();

        let task = tokio::spawn(async move {
            loop {
                match subscriber.recv().await {
                    Ok(event) => handler(event),
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("lag.. {} events dropped", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        self.subscriber_tasks.push(task);
    }
}

impl Drop for EventDispatcher {
    fn drop(&mut self) {
        debug!("dropping EventDispatcher");
        for task in self.subscriber_tasks.drain(..) {
            task.abort();
        }
    }
}
