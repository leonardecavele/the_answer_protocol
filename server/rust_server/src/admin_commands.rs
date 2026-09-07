use tracing::{info, warn};

use crate::{constants::LOST_ITEM, game_manager::GameManager};

impl GameManager {
    pub fn handle_admin_command(&mut self, command: &str) {
        let words: Vec<&str> = command.split_whitespace().collect();

        if let (Some(command), Some(player_name), Some(arg)) = (words.get(0), words.get(1), words.get(2)) {
            match command.to_lowercase().as_str() {
                "giveitem" => {
                    if let Some(player_id) = self.get_player_id(player_name.to_uppercase().as_str()).copied() {
                        let item_name = *arg;
                        let model_id = (0..self.nb_models)
                            .find(|&i| self.get_item(i).is_some_and(|item| item.get_name() == item_name));
                        
                        if let Some(item_id) = model_id {
                            if item_id != LOST_ITEM {
                                let new_item_id = self.instantiate_item(item_id);
                                self.add_item_to_player(player_id, new_item_id);
                            } else {
                                warn!("Cannot give objet_perdu");
                            }
                        } else {
                            warn!("Item not found: {}", item_name);
                        }
                    } else {
                        warn!("Player not found: {}", player_name);
                    }
                }
                _ => warn!("unknown 2 argument admin command: {} (args : {} {})", command, player_name, arg),
            }
        } else if let (Some(command), Some(arg)) = (words.get(0), words.get(1)) {
            match command.to_lowercase().as_str() {
                _ => warn!("unknown 1 argument admin command: {} (arg : {})", command, arg),
            }
        } else if let Some(command) = words.get(0) {
            match command.to_lowercase().as_str() {
                "showitems" => {
                    let all_items: Vec<String> = self.all_items.iter().map(|(_, item)| item.get_protocol_representation()).collect();
                    info!("all items: {:?}", all_items);
                }
                _ => warn!("unknown admin command: {}", command),
            }
        } else {
            info!("admin command not recognized: {}", command);
        }
    }
}
