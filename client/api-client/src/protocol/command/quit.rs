use crate::client::ServerInfo;
use crate::protocol::command::{Command, CommandResult, CreateCommandResult};
use crate::protocol::response::ServerResponse;

pub struct QuitCommand;

pub struct QuitResponse;

impl Command for QuitCommand {
    type ResponseData = QuitResponse;

    fn create_command(&self, server_info: &ServerInfo) -> CreateCommandResult {
        match server_info.protocol_version {
            1 => CreateCommandResult::Success {
                raw_command: "QUIT".to_string(),
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
                if let Some(arguments) = response.arguments {
                    if arguments.len() < 1 || arguments[0] != "bye" {
                        return CommandResult::Error {
                            message: "invalid arguments".to_string(),
                        };
                    }

                    CommandResult::Success {
                        data: QuitResponse,
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
