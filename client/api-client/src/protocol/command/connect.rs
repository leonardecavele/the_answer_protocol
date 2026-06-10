use crate::client::ServerInfo;
use crate::error::{TapError, TapResult};
use crate::protocol::command::{Command, CommandResult};
use crate::protocol::response::{ServerResponse, ServerResponseOpcode};

pub struct ConnectCommand {
    pub player_name: String,
}

pub struct ConnectServerResponseData {
    pub player_name: String,
}

impl Command for ConnectCommand {
    type ResponseData = ConnectServerResponseData;

    fn create_command(&self, server_info: &ServerInfo) -> TapResult<String> {
        match server_info.protocol_version {
            1 => Ok(format!("CONNECT {}", self.player_name)),
            v => todo!(
                "[command connect] Server version {} is not supported yet",
                v
            ),
        }
    }

    fn parse_response_ok(
        &self,
        server_info: &ServerInfo,
        response: ServerResponse,
    ) -> TapResult<CommandResult<Self::ResponseData>> {
        match server_info.protocol_version {
            1 => {
                if let Some(arguments) = response.arguments.clone() {
                    if arguments.len() != 1 || arguments[0] != "connected" {
                        return Err(TapError::ProtocolInvalidArguments(
                            "OK connected".to_string(),
                            response.raw,
                        ));
                    }

                    Ok(CommandResult::Success {
                        data: ConnectServerResponseData {
                            player_name: self.player_name.clone(),
                        },
                        response,
                    })
                } else {
                    Err(TapError::ProtocolInvalidArguments(
                        "OK connected".to_string(),
                        response.raw,
                    ))
                }
            }
            v => Ok(CommandResult::Error {
                message: "server version {} is not supported yet".to_string(),
                response,
            }),
        }
    }
}
