use tracing::{info, warn};

use crate::{constants::LOST_ITEM, game_manager::GameManager, items::Item};

impl GameManager {
    pub fn handle_admin_command(&mut self, command: &str) {
        let mut parts = command.splitn(3, char::is_whitespace);

        let command = parts.next();
        let player_name = parts.next();
        let arg = parts.next();

        if let (Some(command), Some(player_name), Some(arg)) = (command, player_name, arg) {
            match command.to_lowercase().as_str() {
                "giveitem" => {
                    if let Some(player_id) = self
                        .get_player_id(player_name.to_uppercase().as_str())
                        .copied()
                    {
                        let item_name = arg;
                        let model_id = (0..self.nb_models).find(|&i| {
                            self.get_item(i)
                                .is_some_and(|item| item.get_name() == item_name)
                        });

                        if let Some(item_id) = model_id {
                            if item_id != LOST_ITEM {
                                let new_item_id = self.instantiate_item(item_id);
                                let item_repr =
                                    Item::protocol_representation(new_item_id, item_name);
                                self.add_item_to_player(player_id, new_item_id);
                                let event = GameManager::generate_no_player_event_json(
                                    &vec![player_name.to_string()],
                                    "ITEM ADD",
                                    item_repr.as_str(),
                                );
                                self.add_diff_to_tick(event);
                                info!("gave item {} to player {}", item_name, player_name);
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
                "completequest" => {
                    if let Some(player_id) =
                        self.get_player_id(&player_name.to_uppercase()).copied()
                    {
                        let quest_name = arg.to_string();
                        if let Some(quest) = self.get_quest(&quest_name) {
                            let nb_steps = quest.get_nb_steps();
                            if let Some(quest_instance) =
                                self.quest_instances.iter_mut().find(|q| {
                                    q.get_quest_name() == quest_name && q.get_player() == player_id
                                })
                            {
                                quest_instance.set_step(nb_steps);
                                info!("completed quest {} for player {}", quest_name, player_name);
                            } else {
                                warn!("Quest instance not found: {}", quest_name);
                            }
                        } else {
                            warn!("Quest not found: {}", quest_name);
                        }
                    } else {
                        warn!("Player not found: {}", player_name);
                    }
                }
                _ => warn!(
                    "unknown 2 argument admin command: {} (args : {} {})",
                    command, player_name, arg
                ),
            }
        } else if let (Some(command), Some(arg)) = (command, arg) {
            match command.to_lowercase().as_str() {
                "help" | "?" => match arg.to_lowercase().as_str() {
                    "giveitem" => {
                        info!(
                            "Usage: giveitem <player_name> <item_name> - Give an item to a player"
                        );
                    }
                    "completequest" => {
                        info!(
                            "Usage: completequest <player_name> <quest_name> - Instantly complete a quest for a player"
                        );
                    }
                    "showitems" => {
                        info!("Usage: showitems - List all items loaded on the server");
                    }
                    "help" => {
                        info!(
                            "Usage: help [command] - Show general help or details about a specific command"
                        );
                    }
                    _ => warn!("Unknown command '{}' for help", arg),
                },
                _ => warn!(
                    "unknown 1 argument admin command: {} (arg : {})",
                    command, arg
                ),
            }
        } else if let Some(command) = command{
            match command.to_lowercase().as_str() {
                "showitems" => {
                    let all_items: Vec<String> = self
                        .all_items.values().map(|item| item.get_protocol_representation())
                        .collect();
                    info!("all items: {:?}", all_items);
                }
                "help" | "?" => {
                    info!(
                        "\n============================== AVAILABLE ADMIN COMMANDS ==============================\n\
                         • help                                : Show this help message\n\
                         • help <command>                      : Show help for a specific command\n\
                         • showitems                           : List all items loaded on the server\n\
                         • giveitem <player_name> <item_name>  : Give an item to a player\n\
                         • completequest <player> <quest_name> : Instantly complete a quest for a player\n\
                         ========================================================================================"
                    );
                }
                _ => warn!("unknown admin command: {}", command),
            }
        }
    }
}
