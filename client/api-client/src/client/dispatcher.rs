use std::cmp::PartialEq;
use std::collections::HashMap;
use crate::error::CommandError;
use crate::protocol::command::look::LookResponse;
use crate::protocol::response::ServerResponse;
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use std::string::ToString;
use serde_json::Value;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

pub struct EventDispatcher {
    broadcast_sender: broadcast::Sender<ServerResponse>,
    subscriber_tasks: Vec<JoinHandle<()>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "event_name")]
enum EventType {
    NewPlayer,
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    player: String,
    #[serde(flatten)]
    event_type: EventType,
    data: HashMap<String, Value>
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
                    Ok(response) => {
                        let event = match serde_json::from_str::<Event>(response.arguments.join("").as_str()) {
                            Ok(event) => event,
                            Err(e) => {
                                error!("invalid event: {:?}", e);
                                continue;
                            }
                        };
                        info!("DATA = {:?}", event);

                        if event.event_type == EventType::NewPlayer {
                            info!("OUI c'est egal.");
                        }
                        handler(response)
                    }
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
