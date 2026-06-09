pub mod handshake;
pub mod connect;

use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind};

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
    arguments: Option<Vec<String>>
}

impl Packet {
    pub fn parse(frame: String) -> io::Result<Packet> {
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
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "Invalid frame identifier. \
                            expected (OK, EVT, ERR), received '{}'",
                            x
                        ),
                    ));
                }
            },
            None => {
                return Err(Error::new(ErrorKind::Other, "invalid frame"));
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
