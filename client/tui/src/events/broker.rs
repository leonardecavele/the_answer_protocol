use crate::errors::ApplicationError;
use crate::events::types::ApplicationEvent;
use crossterm::event::EventStream;
use futures::StreamExt;
use mpsc::Sender;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{Instant, interval_at};

pub const TICK_RATE: Duration = Duration::from_millis(10);
pub const MAX_EVENTS_BUS: usize = 100;

pub struct EventBroker {
    receiver: mpsc::Receiver<ApplicationEvent>,
    sender: Sender<ApplicationEvent>,
    background_task: JoinHandle<()>,
}

impl EventBroker {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(MAX_EVENTS_BUS);

        let task_sender = sender.clone();

        let background_task = tokio::spawn(async move {
            let mut event_stream = EventStream::new();
            let mut tick_interval = interval_at(Instant::now() + TICK_RATE, TICK_RATE);

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        if task_sender.send(ApplicationEvent::Tick).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(crossterm_event)) = event_stream.next() => {
                        if task_sender.send(ApplicationEvent::Terminal(crossterm_event)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Self {
            receiver,
            sender,
            background_task,
        }
    }

    pub async fn next_event(&mut self) -> Result<ApplicationEvent, ApplicationError> {
        self.receiver
            .recv()
            .await
            .ok_or(ApplicationError::EventChannelClosed)
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
