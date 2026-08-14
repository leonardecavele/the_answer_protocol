use crate::constantes::{
    BASE_COMMAND_RESPONSE, CODE_NL_SEP, CODE_SP_SEP, ErrorCode, LOST_ITEM, LOST_ITEM_SPAWN,
    MAX_TIME_FOR_COMBAT, MIN_DMG_DEALT, NPC_DMG, NPC_MOB,
};
use crate::game_manager::GameManager;
use crate::items::{Item, ItemId};
use crate::npc::NpcId;
use crate::quests::QuestState;
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
        ignore_emitted_by: bool,
    ) -> JsonValue {
        if ignore_emitted_by {
            players.retain(|player| player != emitted_by);
        }
        return object! {
            "players": players.as_slice(),
            "emitted_by": emitted_by,
            "event_name": event_name,
            "data": data
        };
    }

    pub fn generate_no_player_event_json(
        players: &Vec<String>,
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

    pub fn fight_create_command(
        &mut self,
        leader: &str,
        npc_id: NpcId,
        players: Vec<String>,
    ) -> String {
        let leader_id = *self.get_player_id(leader).unwrap();
        let command_name = "FIGHT_CREATE";
        if let Some(_instance) = self
            .combat_instances
            .get_instance_for_player(*self.get_player_id(leader).unwrap())
        {
            return generate_json(leader, command_name, ErrorCode::PlayerNotInCombat, "").dump();
        }

        for player in &players {
            if !self.player_exists(player) {
                return generate_json(leader, command_name, ErrorCode::PlayerNotFound, "").dump();
            }
            if let Some(_instance) = self
                .combat_instances
                .get_instance_for_player(*self.get_player_id(player).unwrap())
            {
                return generate_json(leader, command_name, ErrorCode::PlayerAlreadyInCombat, "")
                    .dump();
            }
        }
        let grouped_players_ids: Vec<u32> = players
            .iter()
            .filter_map(|name| self.get_player_id(name).copied())
            .collect();

        let file_name = self.get_random_test_file_name();
        self.combat_instances.add_instance(
            leader_id,
            npc_id,
            self.get_npc_hp(npc_id).unwrap(),
            grouped_players_ids,
            file_name.clone(),
        );
        let npc_representation = self
            .all_npcs
            .get(&npc_id)
            .unwrap()
            .get_protocol_representation();

        let code: String = std::fs::read_to_string(&file_name).unwrap();
        let code_without_nl_sp = code.replace(" ", CODE_SP_SEP).replace("\n", CODE_NL_SEP);
        let mut players_to_notify = players.clone();
        players_to_notify.push(leader.to_string());
        let args_to_send = object! { "code": code_without_nl_sp,
                                     "time": MAX_TIME_FOR_COMBAT.as_secs(),
                                     "nl_sep": CODE_NL_SEP,
                                     "sp_sep": CODE_SP_SEP,
                                     "npc_id": npc_representation,
                                     "npc_hp": self.get_npc_hp(npc_id).unwrap(),
                                     "npc_max_hp": self.get_npc_max_hp(npc_id).unwrap()}.dump();
        let event = GameManager::generate_no_player_event_json(
            &players_to_notify,
            "FIGHT START",
            args_to_send.as_str(),
        );
        self.add_diff_to_tick(event);

        return generate_json(leader, command_name, ErrorCode::NoError, "FIGHT CREATED").dump();
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
            let leave_diff = self.generate_event_json(&mut lrp, p, "ROOM", "PRESENCE LEAVE", true);
            self.add_diff_to_tick(leave_diff);

            let mut crp = spectators_enter.clone();
            let enter_diff = self.generate_event_json(&mut crp, p, "ROOM", "PRESENCE ENTER", true);
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
        let mut grouped_players = json_object["grouped_players"]
            .members()
            .map(|x| x.as_str().unwrap().to_string())
            .collect::<Vec<String>>();
        let command_name = json_object["command"].as_str().unwrap();
        let data = json_object["data"].as_str().unwrap();

        match command_name {
            "MOVE" => {
                if let Some(_instance) = self
                    .combat_instances
                    .get_instance_for_player(*self.get_player_id(leader).unwrap())
                {
                    return generate_json(
                        leader,
                        command_name,
                        ErrorCode::PlayerAlreadyInCombat,
                        "",
                    )
                    .dump();
                }
                return self.group_command_move(
                    leader.to_string(),
                    command_name,
                    grouped_players,
                    data,
                );
            }
            "FIGHT_CREATE" => {
                let npc_id = match self.verify_combat_target(leader, command_name, data) {
                    Ok(id) => id,
                    Err(json_response) => return json_response,
                };

                // here call a function with leader id and npc id and grouped players
                return self.fight_create_command(leader, npc_id, grouped_players);
            }
            _ => {
                error!("unknown group command: {}", command_name);
                return "".to_string();
            }
        }
    }

    pub fn verify_combat_target(
        &self,
        player_name: &str,
        command_name: &str,
        target_npc: &str,
    ) -> Result<NpcId, String> {
        let player_room = {
            self.get_player_from_name(player_name)
                .unwrap()
                .get_current_room()
        };
        let npc_wrapped = self.parse_npc(target_npc, player_room.to_string());
        if npc_wrapped.is_none() {
            return Err(
                generate_json(player_name, command_name, ErrorCode::NpcNotFound, "").dump(),
            );
        }
        let (npc_id, _) = npc_wrapped.unwrap();
        if !self.npc_is_in_room(npc_id, player_room) {
            return Err(
                generate_json(player_name, command_name, ErrorCode::NpcNotInRoom, "").dump(),
            );
        }
        let npc_type = self.get_npc_type(npc_id);
        if (npc_type & NPC_MOB) == 0 {
            return Err(
                generate_json(player_name, command_name, ErrorCode::NpcNotHostile, "").dump(),
            );
        }
        if self.is_npc_in_combat(npc_id) {
            return Err(
                generate_json(player_name, command_name, ErrorCode::NpcInCombat, "").dump(),
            );
        }
        Ok(npc_id)
    }

    pub fn process_tester_responses(&mut self) -> std::io::Result<()> {
        while let Ok(response) = self.tester_receiver.try_recv() {
            let json = json::parse(&response).unwrap();
            let player = json["player"].as_str().unwrap();
            let npc_id = json["npc_id"].as_u32().unwrap();
            if let Some(player_id) = self.get_player_id(player) {
                let player_success = json["success"].as_bool().unwrap();
                if player_success {
                    self.player_attacks_npc(20, *player_id, npc_id);
                    let response_msg =
                        generate_json(player, "FIGHT ATTACK", ErrorCode::NoError, "SUCCEED").dump();
                    self.send_msg_to_client(response_msg)?;
                } else {
                    let response_msg =
                        generate_json(player, "FIGHT ATTACK", ErrorCode::NoError, "FAIL").dump();
                    self.send_msg_to_client(response_msg)?;
                }
            }
        }
        Ok(())
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
                        true,
                    );
                    self.remove_item_from_player(player_id, LOST_ITEM as ItemId);
                    self.add_item_to_room(room, LOST_ITEM as ItemId);
                    self.add_diff_to_tick(event);
                }
                let player_success = self.get_player_success(player_id);
                // player_success : Option<Option<bool>
                if player_success.is_some() && player_success.unwrap().is_none() {
                    let npc_id = self
                        .combat_instances
                        .get_instance_for_player(player_id)
                        .unwrap()
                        .get_npc_id();
                    self.npc_attacks_player(NPC_DMG, npc_id, player_id);
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
                    self.parse_npc(target_npc, player_room.to_string());
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
                let parsed_item: Option<(ItemId, String)> =
                    self.parse_item(item, player_room.to_string());
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
                    self.generate_event_json(&mut players_to_send, player_name, "TAKE", item, true);
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
                let room_name = player.get_current_room().to_string();
                let item_tuple: Option<(ItemId, String)> =
                    self.parse_item(item, room_name.to_string());
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

                self.remove_item_from_player(player_id, item_id);
                self.add_item_to_room(&room_name, item_id);

                self.start_dropped_at_for_item(item_id);

                let mut players_to_send = self.get_all_players_at_room(room_name.as_str());
                let events_json =
                    self.generate_event_json(&mut players_to_send, player_name, "DROP", item, true);
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
            "FIGHT_CREATE" => {
                let npc_id = match self.verify_combat_target(player_name, command_name, data) {
                    Ok(id) => id,
                    Err(json_response) => return json_response,
                };

                // here call a function with leader id and npc id and grouped players
                return self.fight_create_command(player_name, npc_id, Vec::new());
            }
            "FIGHT_ATTACK" => {
                if let Some(player_instance) = self
                    .combat_instances
                    .get_instance_for_player(*self.get_player_id(player_name).unwrap())
                {
                    let file_name = player_instance.get_assigned_file_name();
                    let npc_id = player_instance.get_npc_id();
                    let sent_code = data;
                    /*check if the code is correct*/
                    self.test_code(file_name, sent_code, player_name, npc_id);
                }

                return generate_json(player_name, command_name, ErrorCode::PlayerNotInCombat, "")
                    .dump();
            }
            "ATTACK" => {
                let npc_id = match self.verify_combat_target(player_name, command_name, data) {
                    Ok(id) => id,
                    Err(json_response) => return json_response,
                };
                let player_id = *self.get_player_id(player_name).unwrap();

                if let Some(instance_player_count) =
                    self.get_nb_players_in_player_instance(player_id)
                {
                    if self.check_action_already_taken(player_id, npc_id) {
                        return generate_json(
                            player_name,
                            command_name,
                            ErrorCode::ActionAlreadyTaken,
                            "",
                        )
                        .dump();
                    }
                    let npc_combat_start_hp = self.get_npc_combat_start_hp(npc_id).unwrap();
                    let npc_hp = self.get_npc_hp(npc_id).unwrap();
                    let mut dmg = (npc_combat_start_hp / instance_player_count).min(MIN_DMG_DEALT);
                    if dmg * 2 > npc_hp {
                        dmg *= 2;
                    }
                    let combat_result = self.player_attacks_npc(dmg, player_id, npc_id);
                    return generate_json(
                        player_name,
                        command_name,
                        ErrorCode::NoError,
                        combat_result.as_str(),
                    )
                    .dump();
                }

                let combat_result = self.player_attacks_npc(NPC_DMG, player_id, npc_id);

                return generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    combat_result.as_str(),
                )
                .dump();
            }
            // "DEFEND" => {
            //     let npc_id = match self.verify_combat_target(player_name, command_name, data) {
            //         Ok(id) => id,
            //         Err(json_response) => return json_response,
            //     };
            //     let player_id = *self.get_player_id(player_name).unwrap();

            //     if let Some(_instance_player_count) =
            //         self.get_nb_players_in_player_instance(player_id)
            //     {
            //         if self.check_action_already_taken(player_id, npc_id) {
            //             return generate_json(
            //                 player_name,
            //                 command_name,
            //                 ErrorCode::ActionAlreadyTaken,
            //                 "",
            //             )
            //             .dump();
            //         }
            //     }

            //     self.npc_attacks_player(NPC_DMG, npc_id, player_id)
            // }
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
                    self.parse_npc(target_npc, player_room.to_string());
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
                    if quests.is_empty() {
                        return generate_json(
                            player_name,
                            command_name,
                            ErrorCode::NoQuestAvailable,
                            "",
                        )
                        .dump();
                    }
                    let mut rng = rand::rng();
                    let random_index = rng.random_range(0..quests.len());
                    if let Some(quest_id) = quests.get(random_index) {
                        let quest_json_str;
                        if let Some(quest) = self.get_quest(quest_id) {
                            let reward = quest.get_json_loots();

                            quest_json_str = json::object! {
                                "quest_id" => quest.get_id().clone(),
                                "description" => quest.get_description(),
                                "reward" => reward,
                                "status" => QuestState::InProgress.to_str()
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
                    .map(|q| {
                        let quest = self.get_quest(&q.get_quest_name()).unwrap();
                        json::object! {
                        "quest_id" => quest.get_id().clone(),
                        "description" => quest.get_description(),
                        "reward" => quest.get_json_loots(),
                        "status" => q.get_state().to_str() }
                    })
                    .collect::<Vec<_>>();
                let quests_json: JsonValue = JsonValue::Array(quests);

                return generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    quests_json.dump().as_str(),
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
