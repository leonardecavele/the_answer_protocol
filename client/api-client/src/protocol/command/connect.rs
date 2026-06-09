use crate::client::ServerInfo;
use crate::error::TapResult;
use crate::protocol::command::Command;

pub fn create_command_connect(server_info: &ServerInfo, player_name: String) -> TapResult<Command> {
    match server_info.protocol_version {
        1 => v1(player_name),
        v => todo!(
            "[command connect] Server version {} is not supported yet",
            v
        ),
    }
}

fn v1(player_name: String) -> TapResult<Command> {
    Ok(Command {
        command: format!("CONNECT {}", player_name),
    })
}
