pub mod greeting;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind};

#[derive(Debug, PartialEq)]
enum PacketType {
    Ok,
    Evt,
    Err,
    Empty,
}

impl Display for PacketType {
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
    packet_type: PacketType,
    arguments: Option<Vec<String>>,
}

impl Packet {
    pub fn new(frame: String) -> io::Result<Packet> {
        let raw_frame = frame.trim().to_owned();
        let frame = frame.trim().to_owned();

        if frame.is_empty() {
            return Ok(Packet {
                raw: raw_frame,
                packet_type: PacketType::Empty,
                arguments: None,
            });
        }

        let frame_type: PacketType = match frame.split(' ').nth(0) {
            Some(x) => match x {
                "OK" => PacketType::Ok,
                "EVT" => PacketType::Evt,
                "ERR" => PacketType::Err,
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
            packet_type: frame_type,
            arguments: (!arguments.is_empty()).then_some(arguments),
        })
    }
}
