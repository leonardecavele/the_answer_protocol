use crate::game_manager::GameManager;
use json::object;
use tracing::{info, error};
use crate::constantes::{ErrorCode, BASE_COMMAND_RESPONSE};
use crate::player::PlayerId;
impl GameManager {
    pub fn handle_message(&mut self, msg: String) -> String {
            /*
            read the message, simulate the corresponding action and return the response
            */
            info!("received message: {}", msg);

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
            // let arguments = json_object["arguments"].unwrap();


            info!("received command: {} from player: {}", command_name, player_name);
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

                // "CHAT" => {},
                "WHO" => {
                    return object!{
                        "player": player_name,
                        "command": command_name,
                        "error_code": ErrorCode::NoError.code(),
                        "value": self.get_nb_players()
                    }.dump();
                },
                "GROUP CREATE" => {
                    let player_id: PlayerId = *self.get_players_by_names().get(player_name).unwrap();
                    let error_type: ErrorCode = self.create_group(player_id);
                    return object!{
                        "player": player_name,
                        "command": command_name,
                        "error_code": error_type.code(),
                        "value": ""
                    }.dump()
                },
                // "GROUP INVITE" => {
                //     let player_to_invite: String = arguments["username"].as_str().unwrap().to_string();

                    
                //                     {
                //     "username": <username>
                // }
                // },
                // "GROUP JOIN" => {},
                // "GROUP LEAVE" => {},
                // "TAKE" => {},
                // "DROP" => {},
                // "INVENTORY" => {},
                // "TALK" => {},
                // "ATTACK" => {},
                // "STATUS" => {},
                // "QUEST" => {},
                // "QUESTS" => {},
                _ => {println!("Unknown command: {}", command_name); return "".to_string();}
            }
        }
    }