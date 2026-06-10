pub mod connect;

use crate::client::ServerInfo;
use crate::error::TapResult;
use crate::protocol::response::ServerResponse;

pub trait Command {
    type Response;

    fn create_command(&self, server_info: &ServerInfo) -> TapResult<String>;

    fn parse_response(
        &self,
        server_info: &ServerInfo,
        response: ServerResponse,
    ) -> TapResult<Self::Response>;
}
