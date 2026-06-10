use crate::client::ServerInfo;
use crate::protocol::command::{Command, CommandResult, CreateCommandResult};
use crate::protocol::response::ServerResponse;

pub struct LookCommand;

pub struct LookServerResponseData {
    pub json_data: String,
}

impl Command for LookCommand {
    type ResponseData = LookServerResponseData;

    fn create_command(&self, server_info: &ServerInfo) -> CreateCommandResult {
        match server_info.protocol_version {
            1 => CreateCommandResult::Success {
                raw_command: "LOOK".to_string(),
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
                    if arguments.len() < 2 {
                        return CommandResult::Error {
                            message: "invalid arguments".to_string(),
                        };
                    }

                    // TD: create HELPER to parse JSON data: arguments[1..].join(" ")

                    CommandResult::Success {
                        data: LookServerResponseData {
                            json_data: arguments[1..].join(" "),
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
