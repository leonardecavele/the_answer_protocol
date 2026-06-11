use crate::error::{TapError, TapResult};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub enum Opcode {
    Ok,
    Evt,
    Err,
    Empty,
}

pub fn server_error_message_from_code(code: i32) -> String {
    match code {
        201 => String::from("username already taken"),
        301 => String::from("no exit available in this direction"),
        401 => String::from("player is not in a group"),
        402 => String::from("player is already in a group"),
        404 => String::from("item, inventory item, or NPC not found"),
        405 => String::from("target NPC is not hostile"),
        406 => String::from("no quest available"),
        900 => String::from("connection failed"),
        901 => String::from("send failed"),
        _ => "unknown server error".to_string(),
    }
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

#[derive(Debug, Clone)]
pub struct ServerResponse {
    pub raw: String,
    pub opcode: Opcode,
    pub arguments: Option<Vec<String>>,
}

impl TryFrom<String> for ServerResponse {
    type Error = TapError;

    fn try_from(frame: String) -> TapResult<ServerResponse> {
        let raw_frame = frame.trim().to_owned();
        let frame = frame.trim().to_owned();

        if frame.is_empty() {
            return Ok(ServerResponse {
                raw: raw_frame,
                opcode: Opcode::Empty,
                arguments: None,
            });
        }

        let frame_type: Opcode = match frame.split(' ').nth(0) {
            Some(x) => match x {
                "OK" => Opcode::Ok,
                "EVT" => Opcode::Evt,
                "ERR" => Opcode::Err,
                _ => {
                    return Err(TapError::ServerResponseParse(format!(
                        "Invalid frame identifier. \
                            expected (OK, EVT, ERR), received '{}'",
                        x
                    )));
                }
            },
            None => {
                return Err(TapError::ServerResponseParse("invalid frame".to_string()));
            }
        };

        let arguments: Vec<String> = frame
            .split(" ")
            .skip(1)
            .map(str::to_owned)
            .collect::<Vec<String>>();

        Ok(ServerResponse {
            raw: raw_frame,
            opcode: frame_type,
            arguments: (!arguments.is_empty()).then_some(arguments),
        })
    }
}
