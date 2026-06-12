use crate::game_manager::GameManager;
use json::object;
use tracing::{info, error};
use crate::constantes::{ErrorCode, BASE_COMMAND_RESPONSE};

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


            info!("received command {} from player {}", command_name, player_name);
            if !arguments.is_null() {
                info!("with arguments {}", arguments);
            }
        
            match command_name {
                "CONNECT" => {
                    self.connect_player(player_name.to_string());
                    return BASE_COMMAND_RESPONSE.to_string();
                }
                "LOOK" => {
                    let harcoded_room = object!{
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
                    return object!{
                        "player": player_name,
                        "command": command_name,
                        "error_code": ErrorCode::NoError.code(),
                        "value": harcoded_room.dump()
                    }.dump();
                },
                // "MOVE" => {},
                "QUIT" => {
                    self.disconnect_player(player_name.to_string());
                    return BASE_COMMAND_RESPONSE.to_string();
                },
                "WHO" => {
                    return object!{
                        "player": player_name,
                        "command": command_name,
                        "error_code": ErrorCode::NoError.code(),
                        "value": self.get_nb_players()
                    }.dump();
                },

                // "TALK" => {},
                // "TAKE" => {},
                // "DROP" => {},
                "INVENTORY" => {
                    let inventory = self.get_player_inventory(player_name);
                    return object! {
                        "player": player_name,
                        "command": command_name,
                        "error_code": ErrorCode::NoError.code(),
                        "value": inventory
                    }.dump();
                },
                // "ATTACK" => {},
                // "STATUS" => {},
                // "QUEST" => {},
                // "QUESTS" => {},
                _ => {println!("Unknown command: {}", command_name); return "".to_string();}
            }
        }
    }