use crate::constantes::{BASE_COMMAND_RESPONSE, ErrorCode};
use crate::game_manager::GameManager;
use crate::items::ItemId;
use crate::room::Room;
use json::{JsonValue, object};
use tracing::{error, info};

fn generate_json(player: &str, command: &str, error_code: ErrorCode, data: &str) -> JsonValue {
    return object! {
        "player": player,
        "command": command,
        "error_code": error_code.code(),
        "data": data // most of the time ""
    };
}

fn generate_question_json(question: &str, data: &str, id: &str) -> JsonValue {
    return object! {
        "question": question,
        "data": data,
        "id": id
    };
}

impl GameManager {
    fn get_item_id_from_name(&self, item_name: &str) -> ItemId {
        let item_id = item_name.split('.').nth(1).unwrap();
        let item_id_int = item_id.parse::<ItemId>().unwrap();
        return item_id_int;
    }

    fn get_item_name_from_id(&mut self, item_id: ItemId) -> String {
        let item_name = self.get_all_items().get(&item_id).unwrap().get_name();
        return format!("item.{}.{}", item_id, item_name);
    }

    fn validate_command_json(&self, parsed_json: &JsonValue) -> ErrorCode {
        if !parsed_json.has_key("command")
            || !parsed_json.has_key("player")
            || !parsed_json.has_key("data")
        {
            error!("invalid json: {}", parsed_json.dump());
            return ErrorCode::InvalidCommand;
        } else {
            return ErrorCode::NoError;
        }
    }

    fn generate_event_json(
        &self,
        players: &Vec<String>,
        ignored_players: &[&str],
        emitted_by: &str,
        event_name: &str,
        data: &str,
    ) -> JsonValue {
        return object! {
            "players": players.as_slice(),
            "ignored_players": ignored_players,
            "emitted_by": emitted_by,
            "event_name": event_name,
            "data": data
        };
    }

    fn validate_question_json(&self, parsed_json: &JsonValue) -> ErrorCode {
        if !parsed_json.has_key("question")
            || !parsed_json.has_key("data")
            || !parsed_json.has_key("id")
        {
            // error!("invalid json: {}", parsed_json.dump());
            return ErrorCode::InvalidQuestion;
        } else {
            return ErrorCode::NoError;
        }
    }

    pub fn handle_server_question(&mut self, parsed_json: &JsonValue) -> String {
        let question: &str = parsed_json["question"].as_str().unwrap();
        let data = parsed_json["data"].as_str().unwrap();
        let id = parsed_json["id"].as_str().unwrap();
        match question {
            "ROOM_PLAYERS" => {
                let player_to_check_name = data;
                let player_to_check = self.get_player_from_name(player_to_check_name);
                let player_to_check_current_room = player_to_check.unwrap().get_current_room();
                let players = self.get_all_players_at_room(player_to_check_current_room);
                return generate_question_json(question, format!("{:?}", players).as_str(), id)
                    .dump();
            }
            _ => {
                error!("unknown question: {}", question);
                return "".to_string();
            }
        }
    }

    pub fn handle_message(&mut self, msg: String) -> String {
        /*
        read the message, simulate the corresponding action and return the response
        */

        let json = json::parse(&msg);

        if json.is_err() {
            error!("invalid json");
            return generate_json("", "", ErrorCode::InvalidCommand, "").dump();
        }

        let json_object = json.unwrap();

        let server_question_json = self.validate_question_json(&json_object);
        if server_question_json == ErrorCode::NoError {
            return self.handle_server_question(&json_object);
        }

        let command_json_validity = self.validate_command_json(&json_object);
        if command_json_validity == ErrorCode::InvalidCommand {
            return generate_json("", "", ErrorCode::InvalidCommand, "").dump();
        }

        let player_name = json_object["player"].as_str().unwrap();

        let command_name = json_object["command"].as_str().unwrap();
        let data = json_object["data"].as_str().unwrap();

        info!(
            "received command {} from player {}",
            command_name, player_name
        );

        match command_name {
            "CONNECT" => {
                // if player_name in self.get_players_by_names().keys(){

                // }
                self.connect_player(player_name.to_string());
                return BASE_COMMAND_RESPONSE.to_string();
            }
            "LOOK" => {
                let room = object! {
                    "room": {
                        "id": "",
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
                return generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    room.dump().as_str(),
                )
                .dump();
            }
            "MOVE" => {
                let direction = data;
                if direction != "NORTH"
                    && direction != "SOUTH"
                    && direction != "EAST"
                    && direction != "WEST"
                {
                    return generate_json(player_name, command_name, ErrorCode::NoExit, "").dump();
                }

                let player = self.get_player_from_name(player_name).unwrap();
                let mut last_room_players = self.get_all_players_at_room(player.get_current_room());
                let room_to_go = {
                    let player = self.get_player_from_name(player_name).unwrap();
                    let current_player_room_name = player.get_current_room();
                    let room_to_go_wrapped = self
                        .get_neighbor_room_name(current_player_room_name, &direction.to_string());
                    if room_to_go_wrapped.is_none() {
                        return generate_json(player_name, command_name, ErrorCode::NoExit, "")
                            .dump();
                    }
                    room_to_go_wrapped.unwrap().clone()
                };

                self.move_player_to_room(player_name, room_to_go.as_str());

                let player = self.get_player_from_name(player_name).unwrap();
                let mut current_room_players =
                    self.get_all_players_at_room(player.get_current_room());

                // tick events part
                current_room_players.retain(|x| *x != player_name);
                last_room_players.retain(|x| *x != player_name);

                let room_leave_diff = self.generate_event_json(
                    &last_room_players,
                    &[player_name],
                    player_name,
                    "ROOM",
                    "LEAVE",
                );
                let room_enter_diff = self.generate_event_json(
                    &current_room_players,
                    &[player_name],
                    player_name,
                    "ROOM",
                    "ENTER",
                );

                info!("before add_diff");
                self.add_diff_to_tick(room_leave_diff);
                self.add_diff_to_tick(room_enter_diff);

                info!("after add_diff");
                return generate_json(player_name, command_name, ErrorCode::NoError, "").dump();
            }

            "QUIT" => {
                self.disconnect_player(player_name.to_string());
                return BASE_COMMAND_RESPONSE.to_string();
            }

            // "TALK" => {},
            // TAKE format : item.global_id.item_type ( ex: "item.12.legendary sword")
            "TAKE" => {
                let (player_id, player_room) = {
                    let player = self.get_player_from_name(player_name).unwrap();
                    (player.get_id(), player.get_current_room().to_string())
                };
                let item = data;
                let item_id = self.get_item_id_from_name(item);

                let room_name: String = {
                    let room: &Room = self.get_room(player_room.as_str()).unwrap();
                    if !room.contains_item(item_id) {
                        return generate_json(
                            player_name,
                            command_name,
                            ErrorCode::ItemNotFound,
                            "",
                        )
                        .dump();
                    }
                    room.get_name().to_string()
                };

                self.remove_item_from_room(&room_name, item_id);
                self.add_item_to_player(player_id, item_id);

                let players_to_send = self.get_all_players_at_room(player_room.as_str());
                let events_json = self.generate_event_json(
                    &players_to_send,
                    &vec![player_name],
                    player_name,
                    "TAKE",
                    "",
                );
                self.add_diff_to_tick(events_json);

                return generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    format!("{}", self.get_item_name_from_id(item_id)).as_str(),
                )
                .dump();
            }
            "DROP" => {
                let player = self.get_player_from_name(player_name).unwrap();
                let player_id = player.get_id();
                let item = data;
                let item_id = self.get_item_id_from_name(item);
                if !self.item_exists(item_id) {
                    return generate_json(
                        player_name,
                        command_name,
                        ErrorCode::ItemNotInInventory,
                        "",
                    )
                    .dump();
                }

                let room_name = player.get_current_room().to_string();
                self.remove_item_from_player(player_id, item_id);
                self.add_item_to_room(&room_name, item_id);

                let players_to_send = self.get_all_players_at_room(room_name.as_str());
                let events_json = self.generate_event_json(
                    &players_to_send,
                    &vec![player_name],
                    player_name,
                    "DROP",
                    "",
                );
                self.add_diff_to_tick(events_json);

                return generate_json(player_name, command_name, ErrorCode::NoError, "").dump();
            }
            "INVENTORY" => {
                let inventory = self.get_player_inventory_as_string(player_name);
                return generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    inventory.as_str(),
                )
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
