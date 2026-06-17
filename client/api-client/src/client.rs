pub mod api;
pub mod connect;
pub mod event;

use crate::client::event::ServerEvent;
use crate::error::{CommandError, InternalError, TapError};
use crate::protocol::command::Command;
use crate::protocol::request::Request;
use crate::protocol::response::Opcode;
use event::EventDispatcher;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tracing::info;
use crate::protocol::command::core::look::LookResponse;

#[derive(Debug)]
pub struct ServerInfo {
    pub addr: String,
    pub protocol_version: u32,
}

#[derive(Debug, Default)]
pub struct GameInfo {
    pub player_name: Option<String>,
    pub room_id: Option<String>,
    pub group_id: Option<String>,
    pub world: Option<LookResponse>
}

struct BridgeState {
    bridge_task: JoinHandle<()>,
    command_sender: Sender<Request>,
}

pub struct Client {
    pub server: ServerInfo,
    pub game: GameInfo,
    bridge: BridgeState,
    event_dispatcher: EventDispatcher,
}

impl Client {
    pub fn on_event<F>(&mut self, handler: F)
    where
        F: FnMut(ServerEvent) + Send + 'static,
    {
        self.event_dispatcher.subscribe(handler);
    }

    pub fn on_connect_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::Connect(name) = event {
                handler(name);
            }
        });
    }

    pub fn on_quit_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::Quit(name) = event {
                handler(name);
            }
        });
    }

    pub fn on_room_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(event::RoomEvent) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::Room(data) = event {
                handler(data);
            }
        });
    }

    pub fn on_group_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(event::GroupEvent) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::Group(data) = event {
                handler(data);
            }
        });
    }

    pub fn on_global_chat_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(event::ChatMessage) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::GlobalChat(data) = event {
                handler(data);
            }
        });
    }

    pub fn on_private_chat_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(event::ChatMessage) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::PrivateChat(data) = event {
                handler(data);
            }
        });
    }

    pub fn on_stats_event<F>(&mut self, mut handler: F)
    where
        F: FnMut(u32) + Send + 'static,
    {
        self.on_event(move |event| {
            if let ServerEvent::Stats(count) = event {
                handler(count);
            }
        });
    }

    async fn request<C: Command>(
        &self,
        command: C,
    ) -> Result<Result<C::ResponseData, CommandError>, TapError> {
        match command.create_command(&self.server) {
            Ok(raw_command) => {
                let (request, response_receiver) = Request::new(raw_command);

                self.bridge
                    .command_sender
                    .send(request)
                    .await
                    .map_err(|_| {
                        InternalError::ChannelPanic(
                            "failed to send command to the bridge task (task may have crashed)"
                                .to_string(),
                        )
                    })?;

                let response = response_receiver.await.map_err(|_| {
                    InternalError::ChannelPanic(
                        "bridge task dropped the response channel without \
                        replying (connection probably died)"
                            .to_string(),
                    )
                })?;

                match response.opcode {
                    Opcode::Ok => Ok(command.parse_response(&self.server, response)),
                    _ => {
                        let mut command_error = CommandError::from_response(response);
                        command.refine_error(&self.server, &mut command_error);
                        Ok(Err(command_error))
                    }
                }
            }
            Err(e) => Ok(Err(e)),
        }
    }

    pub fn is_connected(&self) -> bool {
        !self.bridge.bridge_task.is_finished()
    }

    pub fn close(self) {
        drop(self)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.bridge.bridge_task.abort();
        info!("Api client dropped :: background tasks properly stopped.");
    }
}
