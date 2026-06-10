use crate::error::{TapError, TapResult};
use crate::protocol::response::{ServerResponse, ServerResponseOpcode};
use regex::Regex;
use std::sync::LazyLock;

static RE_TAP_HANDSHAKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"OK hello proto=(?P<proto>\d+)").unwrap());

#[derive(Debug)]
pub struct HandshakeServerResponse {
    pub server_protocol_version: u32,
}

impl TryFrom<ServerResponse> for HandshakeServerResponse {
    type Error = TapError;

    fn try_from(server_response: ServerResponse) -> TapResult<Self> {
        if server_response.opcode != ServerResponseOpcode::Ok {
            return Err(TapError::ProtocolInvalidOpcode(
                ServerResponseOpcode::Ok.to_string(),
                server_response.opcode.to_string(),
            ));
        }

        if let Some(arguments) = server_response.arguments {
            if arguments.len() != 2 || arguments[0] != "hello" {
                return Err(TapError::ProtocolInvalidArguments(
                    "OK hello proto=<unsigned number>".to_string(),
                    server_response.raw,
                ));
            }

            let server_protocol_version: i64 =
                match RE_TAP_HANDSHAKE.captures(&server_response.raw) {
                    Some(caps) => caps.name("proto").unwrap().as_str().to_owned(),
                    None => {
                        return Err(TapError::ProtocolInvalidArguments(
                            "OK hello proto=<unsigned number>".to_string(),
                            server_response.raw,
                        ));
                    }
                }
                .parse()
                .map_err(|_| {
                    TapError::ProtocolInvalidArguments(
                        "OK hello proto=<unsigned number>".to_string(),
                        server_response.raw.clone(),
                    )
                })?;

            if server_protocol_version <= 0 {
                return Err(TapError::ProtocolInvalidArguments(
                    "OK hello proto=<unsigned number>".to_string(),
                    server_response.raw,
                ));
            }

            if server_protocol_version > u32::MAX as i64 {
                return Err(TapError::ProtocolInvalidArguments(
                    "OK hello proto=<unsigned number>".to_string(),
                    server_response.raw,
                ));
            }

            Ok(HandshakeServerResponse {
                server_protocol_version: server_protocol_version as u32,
            })
        } else {
            Err(TapError::ProtocolInvalidArguments(
                "OK hello proto=<unsigned number>".to_string(),
                server_response.raw,
            ))
        }
    }
}
