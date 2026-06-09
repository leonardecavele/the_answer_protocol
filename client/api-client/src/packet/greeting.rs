use crate::packet::{Packet, PacketType};
use regex::Regex;
use std::io;
use std::io::{Error, ErrorKind};
use std::sync::LazyLock;

static RE_TAP_GREETINGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"OK hello proto=(?P<proto>.+)").unwrap());

#[derive(Debug)]
pub struct GreetingPacket {
    pub server_protocol_version: String,
}

impl TryFrom<Packet> for GreetingPacket {
    type Error = Error;

    fn try_from(packet: Packet) -> io::Result<Self> {
        if packet.packet_type != PacketType::Ok {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid greeting packet type. expected (OK), received '{}'",
                    packet.packet_type
                ),
            ));
        }

        if let Some(arguments) = packet.arguments {
            if arguments.len() != 2 || arguments[0] != "hello" {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "Invalid greeting arguments, \
                            expected (OK hello proto=<version>), received '{}'",
                        packet.raw
                    ),
                ));
            }

            let server_protocol_version = match RE_TAP_GREETINGS.captures(&packet.raw) {
                Some(caps) => caps.name("proto").unwrap().as_str().to_owned(),
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        "Invalid greeting protocol version",
                    ));
                }
            };

            Ok(GreetingPacket {
                server_protocol_version,
            })
        } else {
            Err(Error::new(ErrorKind::Other, "invalid greeting"))
        }
    }
}
