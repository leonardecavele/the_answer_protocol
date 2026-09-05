use crate::ClientError;
use client_api::events::ServerEvent;
use client_api::{ApiRequest, ApiResponse, Frame};
use crossterm::event::Event as CrosstermEvent;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::task::JoinHandle;
use tokio::time::{Instant, interval_at};

/// The main event enum that encapsulates all possible events in the application.
#[derive(Debug, Clone)]
pub enum ApplicationEvent {
    DeviceEvent(CrosstermEvent),
    Tick,
    Connection(ConnectionEvent),
    Api(ApiEvent),
    Send(SendEvent),
    FightTimedOut,
}

#[derive(Debug, Clone)]
pub enum SendEvent {
    ApiRequest(ApiRequest),
    RawCommand(String),
}

#[derive(Debug, Clone)]
pub enum ApiEvent {
    ApiResponse {
        response: ApiResponse,
        original_request: ApiRequest,
    },
    Server(ServerEvent),
    Frame(Frame),
    Lagged {
        stream: &'static str,
        count: usize,
    },
    RequestFailed {
        request: ApiRequest,
        error_message: String,
    },
}

/// Events strictly related to the network layer status and data.
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    AttemptStarted {
        server_ip: String,
        server_port: String,
        player_name: String,
    },
    Established {
        server_ip: String,
        server_port: String,
        player_name: String,
    },
    Failed {
        error_message: String,
    },
    Lost {
        reason: String,
    },
}

pub const TICK_RATE: Duration = Duration::from_millis(33);
const MAX_EVENTS_BUS: usize = 200;

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
