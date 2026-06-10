pub mod connect;

use crate::client::ServerInfo;
use crate::error::{TapError, TapResult};
use crate::protocol::response::{ServerErrorMessage, ServerResponse};

pub trait Command {
    type ResponseData;

    fn create_command(&self, server_info: &ServerInfo) -> TapResult<String>;

    fn parse_response_ok(
        &self,
        server_info: &ServerInfo,
        response: ServerResponse,
    ) -> TapResult<CommandResult<Self::ResponseData>>;
}

pub enum CommandResult<T> {
    Success {
        data: T,
        response: ServerResponse,
    },
    Error {
        message: String,
        response: ServerResponse,
    },
}

impl<T> CommandResult<T> {
    pub fn error_from_response(response: ServerResponse) -> CommandResult<T> {
        if let Some(arguments) = response.arguments.clone() {
            if arguments.len() < 2 {
                return CommandResult::Error {
                    message: "failed to retrieve error from the server".to_string(),
                    response,
                };
            }

            match arguments[0].parse::<i32>() {
                Ok(errcode) => {
                    let message_error = ServerErrorMessage::from_code(errcode);

                    if message_error.is_none() {
                        return CommandResult::Error {
                            message: "failed to retrieve error from the server".to_string(),
                            response,
                        };
                    }

                    let message = message_error.unwrap();

                    return CommandResult::Error { message, response };
                }
                Err(_) => {
                    return CommandResult::Error {
                        message: "failed to retrieve error from the server".to_string(),
                        response,
                    };
                }
            }
        }

        CommandResult::Error {
            message: "failed to retrieve error from the server".to_string(),
            response,
        }
    }
}
