use crate::error::ProtocolError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum Opcode {
    Ok,
    Evt,
    Err,
    Empty,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::Err => write!(f, "ERR"),
            Self::Evt => write!(f, "EVT"),
            Self::Empty => write!(f, "<EMPTY>"),
        }
    }
}

impl FromStr for Opcode {
    type Err = ProtocolError;

    fn from_str(s: &str) -> Result<Self, ProtocolError> {
        match s {
            "OK" => Ok(Opcode::Ok),
            "EVT" => Ok(Opcode::Evt),
            "ERR" => Ok(Opcode::Err),
            _ => Err(ProtocolError::InvalidOpcode {
                expected: "(OK, EVT, ERR)".to_string(),
                received: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServerResponse {
    pub raw: String,
    pub opcode: Opcode,
    pub arguments: Vec<String>,
}

impl TryFrom<String> for ServerResponse {
    type Error = ProtocolError;

    fn try_from(frame: String) -> Result<ServerResponse, ProtocolError> {
        let frame = frame.trim();

        if frame.is_empty() {
            return Ok(ServerResponse {
                raw: String::new(),
                opcode: Opcode::Empty,
                arguments: vec![],
            });
        }

        let mut parts = frame.split(" ");

        let opcode_str = parts
            .next()
            .ok_or_else(|| ProtocolError::Parse("invalid frame: missing opcode".to_string()))?;

        let opcode = Opcode::from_str(opcode_str)?;
        let arguments: Vec<String> = parts.map(str::to_owned).collect::<Vec<String>>();

        Ok(ServerResponse {
            raw: frame.to_string(),
            opcode,
            arguments,
        })
    }
}
