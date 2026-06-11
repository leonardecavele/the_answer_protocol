use crate::error::ProtocolError;
use crate::protocol::response::{Opcode, ServerResponse};
use regex::Regex;
use std::sync::LazyLock;

static RE_TAP_HANDSHAKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"OK hello proto=(?P<proto>\d+)").unwrap());

#[derive(Debug)]
pub struct HandshakeResponse {
    pub server_protocol_version: u32,
}

impl TryFrom<ServerResponse> for HandshakeResponse {
    type Error = ProtocolError;

    fn try_from(response: ServerResponse) -> Result<Self, ProtocolError> {
        if response.opcode != Opcode::Ok {
            return Err(ProtocolError::InvalidOpcode {
                expected: Opcode::Ok.to_string(),
                received: response.opcode.to_string(),
            });
        }

        if response.arguments.len() != 2 || response.arguments[0] != "hello" {
            return Err(ProtocolError::InvalidArguments {
                expected: "OK hello proto=<unsigned number>".to_string(),
                received: response.raw,
            });
        }

        let server_protocol_version: i64 = match RE_TAP_HANDSHAKE.captures(&response.raw) {
            Some(caps) => caps.name("proto").unwrap().as_str().to_owned(),
            None => {
                return Err(ProtocolError::InvalidArguments {
                    expected: "OK hello proto=<unsigned number>".to_string(),
                    received: response.raw,
                });
            }
        }
        .parse()
        .map_err(|_| ProtocolError::InvalidArguments {
            expected: "OK hello proto=<unsigned number>".to_string(),
            received: response.raw.clone(),
        })?;

        if server_protocol_version <= 0 {
            return Err(ProtocolError::InvalidArguments {
                expected: "OK hello proto=<unsigned number>".to_string(),
                received: response.raw,
            });
        }

        if server_protocol_version > u32::MAX as i64 {
            return Err(ProtocolError::InvalidArguments {
                expected: "OK hello proto=<unsigned number>".to_string(),
                received: response.raw,
            });
        }

        Ok(HandshakeResponse {
            server_protocol_version: server_protocol_version as u32,
        })
    }
}
