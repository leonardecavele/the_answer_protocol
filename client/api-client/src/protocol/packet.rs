pub mod connect;
pub mod handshake;

use crate::error::{TapError, TapResult};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
enum PacketOpcode {
    Ok,
    Evt,
    Err,
    Empty,
}

impl Display for PacketOpcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::Err => write!(f, "ERR"),
            Self::Evt => write!(f, "EVT"),
            Self::Empty => write!(f, "<EMPTY>"),
        }
    }
}

#[derive(Debug)]
pub struct Packet {
    raw: String,
    opcode: PacketOpcode,
    arguments: Option<Vec<String>>,
}

impl TryFrom<String> for Packet {
    type Error = TapError;

    fn try_from(frame: String) -> TapResult<Packet> {
        let raw_frame = frame.trim().to_owned();
        let frame = frame.trim().to_owned();

        if frame.is_empty() {
            return Ok(Packet {
                raw: raw_frame,
                opcode: PacketOpcode::Empty,
                arguments: None,
            });
        }

        let frame_type: PacketOpcode = match frame.split(' ').nth(0) {
            Some(x) => match x {
                "OK" => PacketOpcode::Ok,
                "EVT" => PacketOpcode::Evt,
                "ERR" => PacketOpcode::Err,
                _ => {
                    return Err(TapError::PacketParse(format!(
                        "Invalid frame identifier. \
                            expected (OK, EVT, ERR), received '{}'",
                        x
                    )));
                }
            },
            None => {
                return Err(TapError::PacketParse("invalid frame".to_string()));
            }
        };

        let arguments: Vec<String> = frame
            .split(" ")
            .skip(1)
            .map(str::to_owned)
            .collect::<Vec<String>>();

        Ok(Packet {
            raw: raw_frame,
            opcode: frame_type,
            arguments: (!arguments.is_empty()).then_some(arguments),
        })
    }
}
