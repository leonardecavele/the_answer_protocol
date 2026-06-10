pub mod connect;
pub mod look;

use crate::client::ServerInfo;
use crate::protocol::response::{ServerResponse, server_error_message_from_code};

pub trait Command {
    type ResponseData;

    fn create_command(&self, server_info: &ServerInfo) -> CreateCommandResult;

    fn parse_response_ok(
        &self,
        server_info: &ServerInfo,
        response: ServerResponse,
    ) -> CommandResult<Self::ResponseData>;
}

pub enum CreateCommandResult {
    Success { raw_command: String },
    Error { message: String },
}

impl CreateCommandResult {
    pub fn server_version_not_implemented_yet(version: u32) -> CreateCommandResult {
        CreateCommandResult::Error {
            message: format!("server version {} is not supported yet", version),
        }
    }
}

pub enum CommandResult<T> {
    Success { data: T },
    Error { message: String },
}

impl<T> CommandResult<T> {
    pub fn error_from_response(response: ServerResponse) -> CommandResult<T> {
        let parsed_arguments = response.arguments.as_ref().and_then(|arguments| {
            if arguments.is_empty() {
                None
            } else {
                let code = arguments[0].parse::<i32>().ok();
                let details = if arguments.len() > 1 {
                    Some(arguments[1..].join(" "))
                } else {
                    None
                };
                Some((code, details))
            }
        });

        let message: String = match parsed_arguments {
            Some((Some(code), details)) => {
                let message = server_error_message_from_code(code);

                match details {
                    Some(details) => {
                        format!("{} (code: {}): {}", message, code, details)
                    }
                    None => {
                        format!("{} (code: {})", message, code)
                    }
                }
            }
            Some((None, Some(details))) => {
                format!(
                    "server returned an invalid error code format. raw details: {}",
                    details
                )
            }
            _ => "failed to retrieve explicit error details from the server".to_string(),
        };

        CommandResult::Error { message }
    }

    pub fn server_version_not_implemented_yet(version: u32) -> CommandResult<T> {
        CommandResult::Error {
            message: format!("server version {} is not supported yet", version),
        }
    }
}
