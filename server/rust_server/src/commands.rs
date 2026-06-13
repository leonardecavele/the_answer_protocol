use crate::constantes::{BASE_COMMAND_RESPONSE, ErrorCode};
use crate::game_manager::GameManager;
use crate::items::ItemId;
use json::object;
use tracing::{error, info};

impl GameManager {
    pub fn handle_message(&mut self, msg: String) -> String {
        /*
        read the message, simulate the corresponding action and return the response
        */

        let json = json::parse(&msg);

        if json.is_err() {
            error!("invalid json: {}", msg);
        }
        let json_object = json.unwrap();
        let player_name_wrapped = json_object["player"].as_str();
        if player_name_wrapped.is_none() {
            error!("invalid json: {}", msg);
        }
        let player_name = player_name_wrapped.unwrap();

        let command_name_wrapped = json_object["command"].as_str();
        if command_name_wrapped.is_none() {
            error!("invalid json: {}", msg);
        }
        let command_name = json_object["command"].as_str().unwrap();
        let arguments = &json_object["arguments"];

        info!(
            "received command {} from player {}",
            command_name, player_name
        );
        if !arguments.is_null() {
            info!("with arguments {}", arguments);
        }

        match command_name {
            "CONNECT" => {
                self.connect_player(player_name.to_string());
                return BASE_COMMAND_RESPONSE.to_string();
            }
            "LOOK" => {
                let harcoded_room = object! {
                    "room": {
                        "id": "room.identifier",
                        "name": "Room Display Name",
                        "description": "Room description text",
                        "exits": {
                        "north": "room.north_id",
                        "south": "room.south_id"
                        }
                    },
                    "players": ["username1", "username2"],
                    "items": ["item.id1", "item.id2"],
                    "npcs": ["npc.id1", "npc.id2"]
                };
                return object! {
                    "player": player_name,
                    "command": command_name,
                    "error_code": ErrorCode::NoError.code(),
                    "value": harcoded_room.dump()
                }
                .dump();
            }
            // "MOVE" => {},
            "QUIT" => {
                self.disconnect_player(player_name.to_string());
                return BASE_COMMAND_RESPONSE.to_string();
            }
            "WHO" => {
                return object! {
                    "player": player_name,
                    "command": command_name,
                    "error_code": ErrorCode::NoError.code(),
                    "value": self.get_nb_players()
                }
                .dump();
            }

            // "TALK" => {},
            // TAKE format : item.global_id.item_type ( ex: "item.12.legendary sword")
            "TAKE" => {
                let player_id = *self.get_players_by_names().get(player_name).unwrap();
                let item = arguments["item_id"].as_str().unwrap();
                let item_id = item.split('.').nth(1).unwrap();
                let item_id_int = item_id.parse::<ItemId>().unwrap();
                if !self.all_items.contains_key(&item_id_int) {
                    return object! {
                        "player": player_name,
                        "command": command_name,
                        "error_code": ErrorCode::ItemNotFound.code(),
                        "value": ""
                    }
                    .dump();
                }

                // change this once rooms are done
                self.all_items.remove(&item_id_int);
                self.add_item_to_player(player_id, item_id_int);

                return object! {
                    "player": player_name,
                    "command": command_name,
                    "error_code": ErrorCode::NoError.code(),
                    "value": ""
                }
                .dump();
            }
            // "DROP" => {},
            "INVENTORY" => {
                let inventory = self.get_player_inventory_as_string(player_name);
                return object! {
                    "player": player_name,
                    "command": command_name,
                    "error_code": ErrorCode::NoError.code(),
                    "value": inventory
                }
                .dump();
            }
            // "ATTACK" => {},
            // "STATUS" => {},
            // "QUEST" => {},
            // "QUESTS" => {},
            _ => {
                println!("Unknown command: {}", command_name);
                return "".to_string();
            }
        }
    }
}
