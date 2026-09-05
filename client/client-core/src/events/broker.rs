use super::ApplicationEvent;
use crate::ClientError;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::task::JoinHandle;
use tokio::time::{Instant, interval_at};

pub const TICK_RATE: Duration = Duration::from_millis(33);
pub const MAX_EVENTS_BUS: usize = 200;

pub struct EventBroker {
    receiver: mpsc::Receiver<ApplicationEvent>,
    sender: Sender<ApplicationEvent>,
    background_task: JoinHandle<()>,
}

impl Default for EventBroker {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBroker {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(MAX_EVENTS_BUS);

        let task_sender = sender.clone();

        let background_task = tokio::spawn(async move {
            let mut tick_interval = interval_at(Instant::now() + TICK_RATE, TICK_RATE);

            loop {
                tick_interval.tick().await;

                if task_sender.send(ApplicationEvent::Tick).await.is_err() {
                    break;
                }
            }
        });

        Self {
            receiver,
            sender,
            background_task,
        }
    }

    pub async fn next_event(&mut self) -> Result<ApplicationEvent, ClientError> {
        self.receiver
            .recv()
            .await
            .ok_or(ClientError::EventChannelClosed)
    }

    pub fn try_next_event(&mut self) -> Result<ApplicationEvent, ClientError> {
        self.receiver.try_recv().map_err(|e| match e {
            TryRecvError::Empty => ClientError::EventChannelEmpty,
            _ => ClientError::EventChannelClosed,
        })
    }

    pub fn sender(&self) -> Sender<ApplicationEvent> {
        self.sender.clone()
    }
}

impl Drop for EventBroker {
    fn drop(&mut self) {
        self.background_task.abort();
    }
}
