use crate::packet::{Packet, PacketOpcode};
use std::io::{Error, ErrorKind};

pub struct ConnectPacket;

impl TryFrom<Packet> for ConnectPacket {
    type Error = Error;

    fn try_from(packet: Packet) -> Result<Self, Self::Error> {
        if packet.opcode != PacketOpcode::Ok {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid connect opcode. expected (OK), received '{}'",
                    packet.opcode
                ),
            ));
        }

        if let Some(arguments) = packet.arguments {
            if arguments.len() != 1 || arguments[0] != "connected" {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "Invalid connect arguments, \
                            expected (OK connected), received '{}'",
                        packet.raw
                    ),
                ));
            }

            Ok(ConnectPacket)
        } else {
            Err(Error::new(ErrorKind::Other, "missing connect arguments"))
        }
    }
}
