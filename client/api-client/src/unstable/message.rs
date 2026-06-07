pub mod event;
pub mod error;

use std::io::{Error, ErrorKind, Result};

pub enum Message {
    Ok(String),
    Err(String),
    Evt(String),
    Empty,
}

impl Message {
    pub fn parse(message: &str) -> Result<Message> {
        if let Some(payload) = message.strip_prefix("OK ") {
            Ok(Message::Ok(payload.to_string()))
        } else if let Some(payload) = message.strip_prefix("EVT ") {
            Ok(Message::Evt(payload.to_string()))
        } else if let Some(payload) = message.strip_prefix("ERR ") {
            Ok(Message::Err(payload.to_string()))
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Invalid message"))
        }
    }
}
