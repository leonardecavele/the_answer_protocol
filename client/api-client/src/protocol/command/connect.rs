use crate::client::ServerInfo;
use crate::protocol::command::{Command, CommandResult, CreateCommandResult};
use crate::protocol::response::ServerResponse;

pub struct ConnectCommand {
    pub player_name: String,
}

pub struct ConnectServerResponseData {
    pub player_name: String,
}

impl Command for ConnectCommand {
    type ResponseData = ConnectServerResponseData;

    fn create_command(&self, server_info: &ServerInfo) -> CreateCommandResult {
        match server_info.protocol_version {
            1 => CreateCommandResult::Success {
                raw_command: format!("CONNECT {}", self.player_name),
            },
            v => CreateCommandResult::server_version_not_implemented_yet(v),
        }
    }

    fn parse_response_ok(
        &self,
        server_info: &ServerInfo,
        response: ServerResponse,
    ) -> CommandResult<Self::ResponseData> {
        match server_info.protocol_version {
            1 => {
                if let Some(arguments) = response.arguments.clone() {
                    if arguments.len() != 1 || arguments[0] != "connected" {
                        return CommandResult::Error {
                            message: "invalid arguments".to_string(),
                        };
                    }

                    CommandResult::Success {
                        data: ConnectServerResponseData {
                            player_name: self.player_name.clone(),
                        },
                    }
                } else {
                    CommandResult::Error {
                        message: "missing arguments".to_string(),
                    }
                }
            }
            v => CommandResult::server_version_not_implemented_yet(v),
        }
    }
}
