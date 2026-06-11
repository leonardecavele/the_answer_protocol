use crate::protocol::response::ServerResponse;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::warn;

pub struct EventDispatcher {
    broadcast_sender: broadcast::Sender<ServerResponse>,
    subscriber_tasks: Vec<JoinHandle<()>>,
}

impl EventDispatcher {
    pub fn new(broadcast_sender: broadcast::Sender<ServerResponse>) -> Self {
        Self {
            broadcast_sender,
            subscriber_tasks: vec![],
        }
    }

    pub fn subscribe<F>(&mut self, handler: F)
    where
        F: Fn(ServerResponse) + Send + 'static,
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

    pub fn shutdown(&mut self) {
        for task in self.subscriber_tasks.drain(..) {
            task.abort();
        }
    }
}
