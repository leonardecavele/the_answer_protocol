use crate::protocol::packet::{Packet, PacketOpcode};
use regex::Regex;
use std::io;
use std::io::{Error, ErrorKind};
use std::sync::LazyLock;

static RE_TAP_HANDSHAKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"OK hello proto=(?P<proto>\d+)").unwrap());

#[derive(Debug)]
pub struct HandshakePacket {
    pub server_protocol_version: u32,
}

impl TryFrom<Packet> for HandshakePacket {
    type Error = Error;

    fn try_from(packet: Packet) -> io::Result<Self> {
        if packet.opcode != PacketOpcode::Ok {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid handshake opcode. expected (OK), received '{}'",
                    packet.opcode
                ),
            ));
        }

        if let Some(arguments) = packet.arguments {
            if arguments.len() != 2 || arguments[0] != "hello" {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "Invalid handshake arguments, \
                            expected (OK hello proto=<version>), received '{}'",
                        packet.raw
                    ),
                ));
            }

            let server_protocol_version: i64 = match RE_TAP_HANDSHAKE.captures(&packet.raw) {
                Some(caps) => caps.name("proto").unwrap().as_str().to_owned(),
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        "Invalid handshake protocol version",
                    ));
                }
            }
            .parse()
            .map_err(|_| {
                Error::new(
                    ErrorKind::InvalidInput,
                    "Handshake server protocol version is not a valid number",
                )
            })?;

            if server_protocol_version <= 0 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Handshake server protocol version is negative",
                ));
            }

            if server_protocol_version > u32::MAX as i64 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Handshake server protocol version is greater than u32::MAX",
                ));
            }

            Ok(HandshakePacket {
                server_protocol_version: server_protocol_version as u32,
            })
        } else {
            Err(Error::new(ErrorKind::Other, "invalid handshake"))
        }
    }
}
