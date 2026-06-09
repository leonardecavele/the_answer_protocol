use std::io;
use crate::client::ServerInfo;
use crate::protocol::command::Command;

pub fn create_command_connect(server_info: &ServerInfo, player_name: String) -> io::Result<Command> {
    let command: String = match server_info.protocol_version {
        version if version > 12 => {
            todo!("[connect] Server version 2.0 is not supported yet")
        }
        _ => format!("CONNECT {}", player_name),
    };

    Ok(Command { command })
}