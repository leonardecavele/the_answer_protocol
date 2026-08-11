pub mod api;
pub mod bridge;
pub mod connect;
pub mod event;

use crate::client::event::ServerEvent;
use crate::error::{CommandError, InternalError, TapError};
use crate::protocol::command::Command;
use crate::protocol::command::enums::{ApiRequest, ApiResponse};
use crate::protocol::request::Request;
use crate::protocol::response::Opcode;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tracing::info;

struct BridgeHandle {
    task: JoinHandle<()>,
    command_sender: Sender<Request>,
    event_sender: broadcast::Sender<ServerEvent>,
}
#[derive(Debug)]
pub struct ServerInfo {
    pub addr: String,
    pub protocol_version: u32,
}
pub struct Client {
    pub server: ServerInfo,
    bridge: BridgeHandle,
}

impl Client {
    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.bridge.event_sender.subscribe()
    }

    async fn request<C: Command>(
        &self,
        command: C,
    ) -> Result<Result<C::ResponseData, CommandError>, TapError> {
        match command.create_command(&self.server) {
            Ok(raw_command) => {
                let (request, response_receiver) = Request::new(raw_command.clone());

                self.bridge
                    .command_sender
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

    pub async fn execute_request(&self, request: ApiRequest) -> Result<ApiResponse, TapError> {
        match request {
            ApiRequest::Connect(cmd) => Ok(ApiResponse::Connect(self.request(cmd).await?)),
            ApiRequest::Quit(cmd) => Ok(ApiResponse::Quit(self.request(cmd).await?)),
            ApiRequest::Look(cmd) => Ok(ApiResponse::Look(self.request(cmd).await?)),
            ApiRequest::Move(cmd) => Ok(ApiResponse::Move(self.request(cmd).await?)),
            ApiRequest::Who(cmd) => Ok(ApiResponse::Who(self.request(cmd).await?)),
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
        !self.bridge.task.is_finished()
    }

    pub fn close(self) {
        drop(self)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.bridge.task.abort();
        info!("Api client dropped :: background tasks properly stopped.");
    }
}
