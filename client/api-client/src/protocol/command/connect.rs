use crate::client::ServerInfo;
use crate::error::{TapError, TapResult};
use crate::protocol::command::Command;
use crate::protocol::response::{ServerResponse, ServerResponseOpcode};

pub struct ConnectCommand {
    pub player_name: String,
}

pub struct ConnectServerResponse;

impl Command for ConnectCommand {
    type Response = ConnectServerResponse;

    fn create_command(&self, server_info: &ServerInfo) -> TapResult<String> {
        match server_info.protocol_version {
            1 => Ok(format!("CONNECT {}", self.player_name)),
            v => todo!(
                "[command connect] Server version {} is not supported yet",
                v
            ),
        }
    }

    fn parse_response(
        &self,
        server_info: &ServerInfo,
        response: ServerResponse,
    ) -> TapResult<Self::Response> {
        match server_info.protocol_version {
            1 => {
                if response.opcode != ServerResponseOpcode::Ok {
                    return Err(TapError::ProtocolInvalidOpcode(
                        ServerResponseOpcode::Ok.to_string(),
                        response.opcode.to_string(),
                    ));
                }

                if let Some(arguments) = response.arguments {
                    if arguments.len() != 1 || arguments[0] != "connected" {
                        return Err(TapError::ProtocolInvalidArguments(
                            "OK connected".to_string(),
                            response.raw,
                        ));
                    }

                    Ok(ConnectServerResponse)
                } else {
                    Err(TapError::ProtocolInvalidArguments(
                        "OK connected".to_string(),
                        response.raw,
                    ))
                }
            }
            v => todo!(
                "[response connect] Server version {} is not supported yet",
                v
            ),
        }
    }
}
