use crate::errors::ApplicationError;
use crate::events::types::ApplicationEvent;
use crossterm::event::EventStream;
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{interval_at, Instant};

pub const TICK_RATE: Duration = Duration::from_millis(500);
pub const MAX_EVENTS_BUS: usize = 100;

/// The EventBroker is responsible for collecting asynchronous events
/// (like terminal inputs and ticks) and funneling them into a single channel.
pub struct EventBroker {
    /// Receiver to consume events in the main application loop.
    receiver: mpsc::Receiver<ApplicationEvent>,
    /// Sender to allow other components (like the network task) to dispatch events.
    sender: mpsc::Sender<ApplicationEvent>,
    /// Handle to the background task listening for terminal events and ticks.
    background_task: JoinHandle<()>,
}

impl EventBroker {
    /// Creates a new EventBroker.
    /// Spawns a Tokio task that will emit `ApplicationEvent::Tick`
    /// at the specified `tick_rate` and `ApplicationEvent::Terminal` for user inputs.
    pub fn new() -> Self {
        // Create an mpsc channel with a reasonable capacity.
        let (sender, receiver) = mpsc::channel(MAX_EVENTS_BUS);

        let task_sender = sender.clone();

        // Spawn the background task
        let background_task = tokio::spawn(async move {
            let mut event_stream = EventStream::new();
            let mut tick_interval = interval_at(
                Instant::now() + TICK_RATE,
                TICK_RATE,
            );

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        if task_sender.send(ApplicationEvent::Tick).await.is_err() {
                            break; // Channel closed, exit task
                        }
                    }
                    Some(Ok(crossterm_event)) = event_stream.next() => {
                        if task_sender.send(ApplicationEvent::Terminal(crossterm_event)).await.is_err() {
                            break; // Channel closed, exit task
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

    /// Waits for the next event to be available.
    /// This method is designed to be called in the main application loop.
    pub async fn next_event(&mut self) -> Result<ApplicationEvent, ApplicationError> {
        self.receiver
            .recv()
            .await
            .ok_or(ApplicationError::EventChannelClosed)
    }

    /// Returns a clone of the sender so that other tasks can send events to the broker.
    pub fn sender(&self) -> mpsc::Sender<ApplicationEvent> {
        self.sender.clone()
    }
}

impl Drop for EventBroker {
    fn drop(&mut self) {
        // Abort the background task when the EventBroker is dropped
        self.background_task.abort();
    }
}
