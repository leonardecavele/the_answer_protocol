
use crate::constantes::{BASE_COMMAND_RESPONSE, ErrorCode, LOST_ITEM, LOST_ITEM_SPAWN, NPC_MOB};
use crate::game_manager::GameManager;
use crate::items::{Item, ItemId};
use crate::npc::{Npc, NpcId};
use crate::room::Room;
use json::{JsonValue, object};
use rand::RngExt;
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
    fn get_item_name_from_id(&mut self, item_id: ItemId) -> String {
        let item_name = self.get_all_items().get(&item_id).unwrap().get_name();
        return format!("{}.{}", item_id, item_name);
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

    pub fn generate_event_json(
        &self,
        players: &mut Vec<String>,
        emitted_by: &str,
        event_name: &str,
        data: &str,
    ) -> JsonValue {
        players.retain(|player| player != emitted_by);
        return object! {
            "players": players.as_slice(),
            "emitted_by": emitted_by,
            "event_name": event_name,
            "data": data
        };
    }

    pub fn generate_no_player_event_json(
        players: &mut Vec<String>,
        event_name: &str,
        data: &str,
    ) -> JsonValue {
        return object! {
            "players": players.as_slice(),
            "event_name": event_name,
            "data": data
        };
    }

    fn validate_grouped_command(&self, parsed_json: &JsonValue) -> ErrorCode {
        if !parsed_json.has_key("leader")
            || !parsed_json.has_key("grouped_players")
            || !parsed_json.has_key("command")
            || !parsed_json.has_key("data")
        {
            // error!("invalid json: {}", parsed_json.dump());
            return ErrorCode::InvalidGroupCommand;
        } else {
            return ErrorCode::NoError;
        }
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

    pub fn group_command_move(
        &mut self,
        leader: String,
        command_name: &str,
        additional_players: Vec<String>,
        direction: &str,
    ) -> String {
        if direction != "NORTH"
            && direction != "SOUTH"
            && direction != "EAST"
            && direction != "WEST"
        {
            return generate_json(&leader, "GROUP", ErrorCode::NoExit, "").dump();
        }

        let player = self.get_player_from_name(&leader).unwrap();
        let last_room_players = self.get_all_players_at_room(player.get_current_room());
        let (room_to_go, room_to_go_id) = {
            let current_player_room_name = player.get_current_room();
            let room_to_go_wrapped =
                self.get_neighbor_room_name(current_player_room_name, &direction.to_string());
            if room_to_go_wrapped.is_none() {
                return generate_json(&leader, command_name, ErrorCode::NoExit, "").dump();
            }
            let room_to_go = room_to_go_wrapped.unwrap().clone();
            let room_id = self.get_room_by_name(room_to_go.as_str()).unwrap().get_id();
            (room_to_go, room_id)
        };

        let mut all_moving_players = vec![leader.clone()];
        all_moving_players.extend(additional_players.clone());

        for p in &all_moving_players {
            self.move_player_to_room(p, room_to_go.as_str());
        }

        let current_room_players = self.get_all_players_at_room(&room_to_go);

        let room_repr = Room::protocol_representation(room_to_go_id, room_to_go.clone());

        let spectators_leave: Vec<String> = last_room_players
            .into_iter()
            .filter(|p| !all_moving_players.contains(p))
            .collect();

        let spectators_enter: Vec<String> = current_room_players
            .into_iter()
            .filter(|p| !all_moving_players.contains(p))
            .collect();

        for p in &all_moving_players {
            let mut lrp = spectators_leave.clone();
            let leave_diff = self.generate_event_json(&mut lrp, p, "ROOM", "LEAVE");
            self.add_diff_to_tick(leave_diff);

            let mut crp = spectators_enter.clone();
            let enter_diff = self.generate_event_json(&mut crp, p, "ROOM", "ENTER");
            self.add_diff_to_tick(enter_diff);

            if p != &leader {
                let move_event = object! {
                    "players": vec![p.as_str()],
                    "emitted_by": leader.as_str(),
                    "event_name": "GROUPMOVE",
                    "data": direction
                };
                self.add_diff_to_tick(move_event);
            }
        }

        return generate_json(
            &leader,
            command_name,
            ErrorCode::NoError,
            room_repr.as_str(),
        )
        .dump();
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

    pub fn handle_group_command(&mut self, json_object: &JsonValue) -> String {
        let leader = json_object["leader"].as_str().unwrap();
        let grouped_players = json_object["grouped_players"]
            .members()
            .map(|x| x.as_str().unwrap().to_string())
            .collect::<Vec<String>>();
        let command_name = json_object["command"].as_str().unwrap();
        let data = json_object["data"].as_str().unwrap();

        match command_name {
            "MOVE" => {
                return self.group_command_move(
                    leader.to_string(),
                    command_name,
                    grouped_players,
                    data,
                );
            }
            _ => {
                error!("unknown group command: {}", command_name);
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

        let group_command_json_validity = self.validate_grouped_command(&json_object);
        if group_command_json_validity == ErrorCode::NoError {
            return self.handle_group_command(&json_object);
        }

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
            "received command {} from player {} with args {}",
            command_name, player_name, data
        );

        match command_name {
            "CONNECT" => {
                // if player_name in self.get_players_by_names().keys(){

                // }
                self.connect_player(player_name.to_string());
                return BASE_COMMAND_RESPONSE.to_string();
            }
            "LOOK" => {
                let (
                    player_room_id,
                    player_room_name,
                    player_room_description,
                    player_room_exits,
                    room_players,
                    room_items_str,
                ) = {
                    let player = self.get_player_from_name(player_name).unwrap();
                    let room = self.get_room_by_name(player.get_current_room()).unwrap();
                    (
                        room.get_id(),
                        room.get_name(),
                        room.get_description(),
                        room.get_exits(),
                        self.get_all_players_at_room(player.get_current_room()),
                        self.convert_items_to_string(room.get_inventory()),
                    )
                };

                let room = object! {
                    "room": {
                        "id": Room::protocol_representation(player_room_id, player_room_name.to_string()),
                        "name": player_room_name,
                        "description": player_room_description,
                        "exits": JsonValue::from(player_room_exits.clone())
                    },
                    "players": JsonValue::from(room_players),
                    "items": JsonValue::from(room_items_str),
                    "npcs": JsonValue::from(self.get_npcs_in_room_as_protocol_representations(player_room_name))
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
                return self.group_command_move(
                    player_name.to_string(),
                    command_name,
                    vec![],
                    data,
                );
            }

            "QUIT" => {
                let player_id = *self.get_player_id(player_name).unwrap();
                let room = LOST_ITEM_SPAWN;
                // this part is for the lost item, who drops when quitting
                if self.player_has_item(player_id, LOST_ITEM as ItemId) {
                    let mut players = self.get_all_players_at_room(room);
                    let lost_item_name = self.get_item_name(&(LOST_ITEM as ItemId));
                    let event = self.generate_event_json(
                        &mut players,
                        player_name,
                        "DROP",
                        Item::protocol_representation(LOST_ITEM as ItemId, lost_item_name.as_str())
                            .as_str(),
                    );
                    self.remove_item_from_player(player_id, LOST_ITEM as ItemId);
                    self.add_item_to_room(room, LOST_ITEM as ItemId);
                    self.add_diff_to_tick(event);
                }

                self.disconnect_player(player_name.to_string());
                return BASE_COMMAND_RESPONSE.to_string();
            }

            "TALK" => {
                let target_npc = data;
                let player_room = {
                    self.get_player_from_name(player_name)
                        .unwrap()
                        .get_current_room()
                };
                let parsed_repr: Option<(NpcId, String)> =
                    Npc::parse_protocol_representation(target_npc);
                if parsed_repr.is_none() {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotFound, "")
                        .dump();
                }
                let (npc_id, npc_name) = parsed_repr.unwrap();
                let npc = self.get_npc(npc_id);
                if npc.is_none() {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotFound, "")
                        .dump();
                }

                let npc_unwrap = npc.unwrap().clone();
                if npc_unwrap.get_name() != npc_name {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotFound, "")
                        .dump();
                }
                if !self.npc_is_in_room(npc_id, player_room) {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotInRoom, "")
                        .dump();
                }

                let dialog = {
                    let player = self.get_mut_player_from_name(player_name).unwrap();
                    player.talk_with(&npc_unwrap)
                };
                return generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    dialog.as_str(),
                )
                .dump();
            }
            // TAKE format : global_id.item_type ( ex: "12.legendary sword")
            "TAKE" => {
                let (player_id, player_room) = {
                    let player = self.get_player_from_name(player_name).unwrap();
                    (player.get_id(), player.get_current_room().to_string())
                };
                let item = data;
                let parsed_item: Option<(ItemId, String)> = Item::parse_item(item);
                if parsed_item.is_none() {
                    return generate_json(player_name, command_name, ErrorCode::ItemNotFound, "")
                        .dump();
                }
                let (item_id, item_name) = parsed_item.unwrap();

                let room_name: String = {
                    let room: &Room = self.get_room_by_name(player_room.as_str()).unwrap();
                    if !self.item_exists_with_name(item_id, item_name.as_str())
                        || !room.contains_item(item_id)
                    {
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
                self.reset_dropped_at_for_item(item_id);

                let mut players_to_send = self.get_all_players_at_room(player_room.as_str());
                let events_json =
                    self.generate_event_json(&mut players_to_send, player_name, "TAKE", item);
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
                let item_tuple: Option<(ItemId, String)> = Item::parse_item(item);
                if item_tuple.is_none() {
                    return generate_json(player_name, command_name, ErrorCode::ItemNotFound, "")
                        .dump();
                }

                let (item_id, item_name) = item_tuple.unwrap();
                if !self.item_exists_with_name(item_id, item_name.as_str())
                    || !player.has_item(item_id)
                {
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

                //set the 2 minutes timer when we drop the item
                
                self.start_dropped_at_for_item(item_id);
                
                let mut players_to_send = self.get_all_players_at_room(room_name.as_str());
                let events_json =
                    self.generate_event_json(&mut players_to_send, player_name, "DROP", item);
                self.add_diff_to_tick(events_json);

                return generate_json(player_name, command_name, ErrorCode::NoError, item).dump();
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
            "ATTACK" => {
                let player_id = self.get_player_id(player_name);
                let target_npc = data;
                let npc_wrapped = Npc::parse_protocol_representation(target_npc);
                let player_room = {
                    self.get_player_from_name(player_name)
                        .unwrap()
                        .get_current_room()
                };
                if npc_wrapped.is_none() {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotFound, "")
                        .dump();
                }
                let (npc_id, _) = npc_wrapped.unwrap();
                if !self.npc_is_in_room(npc_id, player_room) {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotInRoom, "")
                        .dump();
                }
                let npc_type = self.get_npc_type(npc_id);
                if (npc_type & NPC_MOB) == 0 {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotHostile, "")
                        .dump();
                }
                let combat_result = self.player_attacks_npc(*player_id.unwrap(), npc_id);

                return generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    combat_result.as_str(),
                )
                .dump();
            }
            "STATUS" => {
                let player_status = self.get_player_status_as_string(player_name);
                return generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    player_status.as_str(),
                )
                .dump();
            }
            "QUEST" => {
                let target_npc = data;
                let player_room = {
                    self.get_player_from_name(player_name)
                        .unwrap()
                        .get_current_room()
                };
                let parsed_repr: Option<(NpcId, String)> =
                    Npc::parse_protocol_representation(target_npc);
                if parsed_repr.is_none() {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotFound, "")
                        .dump();
                }
                let (npc_id, npc_name) = parsed_repr.unwrap();
                let npc = self.get_npc(npc_id);
                if npc.is_none() {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotFound, "")
                        .dump();
                }

                let npc_unwrap = npc.unwrap().clone();
                if npc_unwrap.get_name() != npc_name {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotFound, "")
                        .dump();
                }
                if !self.npc_is_in_room(npc_id, player_room) {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotInRoom, "")
                        .dump();
                }

                if let Some(mut quests) = npc_unwrap.get_quests().cloned() {
                    let player_id = *self.get_player_id(player_name).unwrap();
                    quests.retain(|quest| !self.player_has_quest(player_id, quest.clone()));
                    let mut rng = rand::rng();
                    let random_index = rng.random_range(0..quests.len());
                    if let Some(quest_id) = quests.get(random_index) {
                        let quest_json_str;
                        if let Some(quest) = self.get_quest(quest_id) {
                            let mut rewards_json = Vec::new();
                            for loot in quest.get_loots() {
                                rewards_json.push(json::object! {
                                    "qty" => loot.qty,
                                    "chance" => loot.chance,
                                    "type" => loot.loot_type.to_string()
                                });
                            }

                            quest_json_str = json::object! {
                                "quest_id" => quest.get_id().clone(),
                                "description" => quest.get_description(),
                                "reward" => rewards_json,
                                "status" => crate::quests::QuestState::InProgress.to_str()
                            }
                            .dump();
                        } else {
                            return generate_json(
                                player_name,
                                command_name,
                                ErrorCode::NoQuestAvailable,
                                "",
                            )
                            .dump();
                        }

                        let player_id = *self.get_player_id(player_name).unwrap();
                        let quest_instance = crate::quests::QuestInstance::new(
                            player_id,
                            quest_id.clone(),
                            crate::quests::QuestState::InProgress,
                        );
                        self.quest_instances.push(quest_instance);

                        return generate_json(
                            player_name,
                            command_name,
                            ErrorCode::NoError,
                            quest_json_str.as_str(),
                        )
                        .dump();
                    }
                }
                return generate_json(player_name, command_name, ErrorCode::NoQuestAvailable, "")
                    .dump();
            }
            "QUESTS" => {
                let player_id = *self.get_player_id(player_name).unwrap();
                let quests = self
                    .quest_instances
                    .iter()
                    .filter(|q| q.get_player() == player_id)
                    .map(|q| (q.get_quest_name(), q.get_state()))
                    .collect::<Vec<_>>();
                return generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    format!("{:?}", quests).as_str(),
                )
                .dump();
            }
            _ => {
                println!("Unknown command: {}", command_name);
                return "".to_string();
            }
        }
    }
}
