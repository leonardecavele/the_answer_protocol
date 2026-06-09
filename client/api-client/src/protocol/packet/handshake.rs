use crate::error::{TapError, TapResult};
use crate::protocol::packet::{Packet, PacketOpcode};
use regex::Regex;
use std::sync::LazyLock;

static RE_TAP_HANDSHAKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"OK hello proto=(?P<proto>\d+)").unwrap());

#[derive(Debug)]
pub struct HandshakePacket {
    pub server_protocol_version: u32,
}

impl TryFrom<Packet> for HandshakePacket {
    type Error = TapError;

    fn try_from(packet: Packet) -> TapResult<Self> {
        if packet.opcode != PacketOpcode::Ok {
            return Err(TapError::ProtocolInvalidOpcode(
                PacketOpcode::Ok.to_string(),
                packet.opcode.to_string(),
            ));
        }

        if let Some(arguments) = packet.arguments {
            if arguments.len() != 2 || arguments[0] != "hello" {
                return Err(TapError::ProtocolInvalidArguments(
                    "OK hello proto=<unsigned number>".to_string(),
                    packet.raw,
                ));
            }

            let server_protocol_version: i64 = match RE_TAP_HANDSHAKE.captures(&packet.raw) {
                Some(caps) => caps.name("proto").unwrap().as_str().to_owned(),
                None => {
                    return Err(TapError::ProtocolInvalidArguments(
                        "OK hello proto=<unsigned number>".to_string(),
                        packet.raw,
                    ));
                }
            }
            .parse()
            .map_err(|_| {
                TapError::ProtocolInvalidArguments(
                    "OK hello proto=<unsigned number>".to_string(),
                    packet.raw.clone(),
                )
            })?;

            if server_protocol_version <= 0 {
                return Err(TapError::ProtocolInvalidArguments(
                    "OK hello proto=<unsigned number>".to_string(),
                    packet.raw,
                ));
            }

            if server_protocol_version > u32::MAX as i64 {
                return Err(TapError::ProtocolInvalidArguments(
                    "OK hello proto=<unsigned number>".to_string(),
                    packet.raw,
                ));
            }

            Ok(HandshakePacket {
                server_protocol_version: server_protocol_version as u32,
            })
        } else {
            Err(TapError::ProtocolInvalidArguments(
                "OK hello proto=<unsigned number>".to_string(),
                packet.raw,
            ))
        }
    }
}
