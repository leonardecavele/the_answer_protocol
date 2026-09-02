use tracing::info;

use crate::game_manager::GameManager;

impl GameManager {
    pub fn handle_admin_command(&mut self, command: &str) {
        let words: Vec<&str> = command.split(' ').collect();

        if let (Some(command), Some(player), Some(arg)) = (words.get(0), words.get(1), words.get(2)) {
            info!("admin command: {} {} {}", command, player, arg);
        } else {
            info!("admin command not recognized: {}", command);
        }
    }
}