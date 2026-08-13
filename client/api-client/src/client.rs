pub(crate) mod api;
pub(crate) mod bridge;
pub(crate) mod connect;
pub(crate) mod event;

use crate::client::event::ServerEvent;
use crate::error::{CommandError, InternalError, TapError};
use crate::protocol::command::enums::{ApiRequest, ApiResponse};
use crate::protocol::command::Command;
use crate::protocol::request::Request;
use crate::protocol::response::Opcode;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::{broadcast, watch};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Connected,
    Closed,
    Lost(String),
}

struct BridgeHandle {
    task: JoinHandle<()>,
    request_sender: Sender<Request>,
    event_sender: broadcast::Sender<ServerEvent>,
    cancellation: CancellationToken,
}
#[derive(Debug)]
pub struct ServerInfo {
    pub addr: String,
    pub protocol_version: u32,
}
pub struct Client {
    pub server: ServerInfo,
    bridge: BridgeHandle,
    close_timeout: Duration,
    state: watch::Receiver<ConnectionState>,
}

pub struct ClientConfig {
    pub connect_timeout: Duration,
    pub handshake_timeout: Duration,
    pub request_timeout: Duration,
    pub close_timeout: Duration,
    pub max_frame_length: usize,
    pub command_channel_capacity: usize,
    pub event_channel_capacity: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            connect_timeout: Duration::from_secs(5),
            handshake_timeout: Duration::from_secs(2),
            request_timeout: Duration::from_secs(10),
            close_timeout: Duration::from_secs(2),
            max_frame_length: 65536,
            command_channel_capacity: 2048,
            event_channel_capacity: 2048,
        }
    }
}

impl Client {
    pub fn state(&self) -> watch::Receiver<ConnectionState> {
        self.state.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.bridge.event_sender.subscribe()
    }

    async fn request<C: Command>(
        &self,
        command: C,
    ) -> Result<Result<C::ResponseData, CommandError>, TapError> {
        let raw_command = command.encode();

        let (request, response_receiver) = Request::new(raw_command.clone());

        self.bridge
            .request_sender
            .send(request)
            .await
            .map_err(|_| {
                InternalError::BridgeUnavailable(format!(
                    "cannot send '{}': the connection to {} is no longer running",
                    raw_command, self.server.addr
                ))
            })?;

        let request_result = response_receiver.await.map_err(|_| {
            InternalError::BridgeUnavailable(format!(
                "no response to '{}': the connection to {} dropped the command \
                        (server disconnected, or it replied with an unreadable frame)",
                raw_command, self.server.addr
            ))
        })?;

        let response = request_result?;

        match response.opcode {
            Opcode::Ok => Ok(command.parse_response(response)),
            _ => {
                let mut command_error = CommandError::from_response(response);
                command.refine_error(&mut command_error);
                Ok(Err(command_error))
            }
        }
    }

    pub async fn execute_request(&self, request: ApiRequest) -> Result<ApiResponse, TapError> {
        match request {
            ApiRequest::Connect(cmd) => Ok(ApiResponse::Connect(self.request(cmd).await?)),
            ApiRequest::Quit(cmd) => Ok(ApiResponse::Quit(self.request(cmd).await?)),
            ApiRequest::Look(cmd) => Ok(ApiResponse::Look(self.request(cmd).await?)),
            ApiRequest::Move(cmd) => Ok(ApiResponse::Move(self.request(cmd).await?)),
            ApiRequest::Who(cmd) => Ok(ApiResponse::Who(self.request(cmd).await?)),
            ApiRequest::FightCreate(cmd) => Ok(ApiResponse::FightCreate(self.request(cmd).await?)),
            ApiRequest::FightAttack(cmd) => Ok(ApiResponse::FightAttack(self.request(cmd).await?)),
            ApiRequest::GlobalChat(cmd) => Ok(ApiResponse::GlobalChat(self.request(cmd).await?)),
            ApiRequest::PrivateChat(cmd) => Ok(ApiResponse::PrivateChat(self.request(cmd).await?)),
            ApiRequest::Take(cmd) => Ok(ApiResponse::Take(self.request(cmd).await?)),
            ApiRequest::Drop(cmd) => Ok(ApiResponse::Drop(self.request(cmd).await?)),
            ApiRequest::Inventory(cmd) => Ok(ApiResponse::Inventory(self.request(cmd).await?)),
            ApiRequest::Status(cmd) => Ok(ApiResponse::Status(self.request(cmd).await?)),
            ApiRequest::Talk(cmd) => Ok(ApiResponse::Talk(self.request(cmd).await?)),
            ApiRequest::Attack(cmd) => Ok(ApiResponse::Attack(self.request(cmd).await?)),
            ApiRequest::Quest(cmd) => Ok(ApiResponse::Quest(self.request(cmd).await?)),
            ApiRequest::Quests(cmd) => Ok(ApiResponse::Quests(self.request(cmd).await?)),
            ApiRequest::GroupCreate(cmd) => Ok(ApiResponse::GroupCreate(self.request(cmd).await?)),
            ApiRequest::GroupJoin(cmd) => Ok(ApiResponse::GroupJoin(self.request(cmd).await?)),
            ApiRequest::GroupLeave(cmd) => Ok(ApiResponse::GroupLeave(self.request(cmd).await?)),
            ApiRequest::GroupInvite(cmd) => Ok(ApiResponse::GroupInvite(self.request(cmd).await?)),
        }
    }

    pub fn is_connected(&self) -> bool {
        matches!(*self.state.borrow(), ConnectionState::Connected)
    }

    pub async fn close(mut self) {
        self.bridge.cancellation.cancel();

        if timeout(self.close_timeout, &mut self.bridge.task)
            .await
            .is_err()
        {
            warn!(
                "bridge did not shut down within {}s, falling back to abort",
                self.close_timeout.as_secs()
            );
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.bridge.task.abort();
        info!("Api client dropped :: background tasks properly stopped.");
    }
}
