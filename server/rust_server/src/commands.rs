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
                "GROUP INVITE" => {
                    let player_to_invite: String = arguments["username"].as_str().unwrap().to_string();

                    let mut base_response = object!{
                        "player": player_name,
                        "command": command_name,
                        "error_code": ErrorCode::NoError.code(),
                        "value": ""
                    };
                    
                    // remove when the go server will checks this
                    if !self.get_players_by_names().contains_key(&player_to_invite) {
                        let _ = base_response.insert("error_code", ErrorCode::NoSuchUser.code());
                        return base_response.dump();
                    }
                    

                    let player_id: PlayerId = *self.get_players_by_names().get(&player_to_invite).unwrap();
                    let player_group_id = self.get_players().get(&player_id).unwrap().get_group_id();
                    if player_group_id.is_some() {
                        let _ = base_response.insert("error_code", ErrorCode::AlreadyInGroup.code());
                        return base_response.dump();
                    }

                    let leader_id: PlayerId = *self.get_players_by_names().get(player_name).unwrap();
                    let leader_group_id_wrapped = self.get_players().get(&leader_id).unwrap().get_group_id();
                    if leader_group_id_wrapped.is_none() {
                        let _ = base_response.insert("error_code", ErrorCode::NotInGroup.code());
                        return base_response.dump();
                    }

                    let leader_group_id = leader_group_id_wrapped.unwrap();
                    let group_leader_id = self.all_groups().get_group(leader_group_id).unwrap().get_leader();
                    if group_leader_id != leader_id {
                        let _ = base_response.insert("error_code", ErrorCode::NotGroupLeader.code());
                        return base_response.dump();
                    }


                    // self.add_pending_invitation(leader_id, player_id)
                    return base_response.dump();
                },
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