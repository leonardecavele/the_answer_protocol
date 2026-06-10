use crate::error::{TapError, TapResult};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum ServerResponseOpcode {
    Ok,
    Evt,
    Err,
    Empty,
}

pub struct ServerErrorMessage;

impl ServerErrorMessage {
    pub fn from_code(code: i32) -> Option<String> {
        match code {
            201 => Some("Requested username already taken".to_string()),
            _ => None,
        }
    }
}

impl Display for ServerResponseOpcode {
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
pub struct ServerResponse {
    pub raw: String,
    pub opcode: ServerResponseOpcode,
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
                opcode: ServerResponseOpcode::Empty,
                arguments: None,
            });
        }

        let frame_type: ServerResponseOpcode = match frame.split(' ').nth(0) {
            Some(x) => match x {
                "OK" => ServerResponseOpcode::Ok,
                "EVT" => ServerResponseOpcode::Evt,
                "ERR" => ServerResponseOpcode::Err,
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
