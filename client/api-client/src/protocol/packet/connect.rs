use crate::client::ServerInfo;
use crate::error::{TapError, TapResult};
use crate::protocol::packet::{Packet, PacketOpcode};

pub struct ConnectPacket;

impl ConnectPacket {
    pub fn parse(server_info: &ServerInfo, packet: Packet) -> TapResult<Self> {
        match server_info.protocol_version {
            1 => Self::v1(packet),
            v => todo!("[packet connect] Server version {} is not supported yet", v),
        }
    }

    fn v1(packet: Packet) -> TapResult<Self> {
        if packet.opcode != PacketOpcode::Ok {
            return Err(TapError::ProtocolInvalidOpcode(
                PacketOpcode::Ok.to_string(),
                packet.opcode.to_string(),
            ));
        }

        if let Some(arguments) = packet.arguments {
            if arguments.len() != 1 || arguments[0] != "connected" {
                return Err(TapError::ProtocolInvalidArguments(
                    "OK connected".to_string(),
                    packet.raw,
                ));
            }

            Ok(ConnectPacket)
        } else {
            Err(TapError::ProtocolInvalidArguments(
                "OK connected".to_string(),
                packet.raw,
            ))
        }
    }
}
