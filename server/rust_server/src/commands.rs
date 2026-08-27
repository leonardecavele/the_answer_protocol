use crate::constantes::{
    BASE_COMMAND_RESPONSE, CODE_NL_SEP, CODE_SP_SEP, ErrorCode, LOST_ITEM, LOST_ITEM_SPAWN,
    MAX_TIME_FOR_COMBAT, NPC_MOB, TEST_FILES_DIR,
};
use crate::game_manager::GameManager;
use crate::items::{Item, ItemId};
use crate::npc::NpcId;
use crate::quests::QuestState;
use crate::room::Room;
use json::{JsonValue, object};
use rand::RngExt;
use tracing::{debug, error, info, warn};

pub fn generate_json(player: &str, command: &str, error_code: ErrorCode, data: &str) -> JsonValue {
    object! {
        "player": player,
        "command": command,
        "error_code": error_code.code(),
        "data": data // most of the time ""
    }
}

fn generate_question_json(question: &str, data: &str, id: &str) -> JsonValue {
    object! {
        "question": question,
        "data": data,
        "id": id
    }
}

impl GameManager {
    fn get_item_name_from_id(&mut self, item_id: ItemId) -> String {
        let item_wrap = self.get_all_items().get(&item_id);
        if item_wrap.is_none() {
            warn!("No item found for item_id: {}", item_id);
            return format!("{}.item_not_found", item_id);
        }
        format!("{}.{}", item_id, item_wrap.unwrap().get_name())
    }

    fn validate_command_json(&self, parsed_json: &JsonValue) -> ErrorCode {
        if !parsed_json.has_key("command")
            || !parsed_json.has_key("player")
            || !parsed_json.has_key("data")
        {
            error!("invalid json: {}", parsed_json.dump());
            ErrorCode::InvalidCommand
        } else {
            ErrorCode::NoError
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
        object! {
            "players": players.as_slice(),
            "emitted_by": emitted_by,
            "event_name": event_name,
            "data": data
        }
    }

    pub fn generate_no_player_event_json(
        players: &Vec<String>,
        event_name: &str,
        data: &str,
    ) -> JsonValue {
        object! {
            "players": players.as_slice(),
            "event_name": event_name,
            "data": data
        }
    }

    fn validate_grouped_command(&self, parsed_json: &JsonValue) -> ErrorCode {
        if !parsed_json.has_key("leader")
            || !parsed_json.has_key("grouped_players")
            || !parsed_json.has_key("command")
            || !parsed_json.has_key("data")
        {
            // error!("invalid json: {}", parsed_json.dump());
            ErrorCode::InvalidGroupCommand
        } else {
            ErrorCode::NoError
        }
    }

    fn validate_question_json(&self, parsed_json: &JsonValue) -> ErrorCode {
        if !parsed_json.has_key("question")
            || !parsed_json.has_key("data")
            || !parsed_json.has_key("id")
        {
            // error!("invalid json: {}", parsed_json.dump());
            ErrorCode::InvalidQuestion
        } else {
            ErrorCode::NoError
        }
    }

    pub fn fight_create_command(
        &mut self,
        leader: &str,
        npc_id: NpcId,
        players: Vec<String>,
    ) -> String {
        let leader_id = match self.get_player_id(leader) {
            Some(id) => *id,
            None => {
                warn!("Leader not found: {}", leader);
                return generate_json(leader, "FIGHT_CREATE", ErrorCode::PlayerNotFound, "").dump();
            }
        };
        let command_name = "FIGHT_CREATE";
        if let Some(_instance) = self.combat_instances.get_instance_for_player(leader_id) {
            return generate_json(leader, command_name, ErrorCode::PlayerAlreadyInCombat, "")
                .dump();
        }

        for player in &players {
            let player_id = match self.get_player_id(player) {
                Some(id) => *id,
                None => {
                    return generate_json(leader, command_name, ErrorCode::PlayerNotFound, "")
                        .dump();
                }
            };
            if let Some(_instance) = self.combat_instances.get_instance_for_player(player_id) {
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
        let npc_representation = match self.all_npcs.get(&npc_id) {
            Some(npc) => npc.get_protocol_representation(),
            None => {
                warn!("NPC not found in all_npcs for id: {}", npc_id);
                return generate_json(leader, command_name, ErrorCode::NpcNotFound, "").dump();
            }
        };

        let file_path = format!("{}/{}", TEST_FILES_DIR, file_name);
        let code: Result<String, std::io::Error> = std::fs::read_to_string(&file_path);
        if code.is_err() {
            error!("Failed to read code from file: {:?}", file_name);
            return generate_json(leader, command_name, ErrorCode::FileNotFound, "").dump();
        }
        let code_without_nl_sp = code
            .unwrap()
            .replace(" ", CODE_SP_SEP)
            .replace("\n", CODE_NL_SEP);
        let mut players_to_notify = players.clone();
        players_to_notify.push(leader.to_owned());
        let args_to_send = object! { "code": code_without_nl_sp,
        "time": MAX_TIME_FOR_COMBAT.as_secs(),
        "nl_sep": CODE_NL_SEP,
        "sp_sep": CODE_SP_SEP,
        "npc_id": npc_representation,
        "npc_hp": self.get_npc_hp(npc_id).unwrap_or_else(|| {warn!("No NPC HP for npc_id: {}", npc_id); 0}),
        "npc_max_hp": self.get_npc_max_hp(npc_id).unwrap_or_else(|| {warn!("No NPC MAX HP for npc_id: {}", npc_id); 0})}
        .dump();
        let event = GameManager::generate_no_player_event_json(
            &players_to_notify,
            "FIGHT START",
            args_to_send.as_str(),
        );
        self.add_diff_to_tick(event);

        generate_json(leader, command_name, ErrorCode::NoError, "FIGHT CREATED").dump()
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

        let player = match self.get_player_from_name(&leader) {
            Some(p) => p,
            None => {
                warn!("Player not found: {}", leader);
                return generate_json(&leader, command_name, ErrorCode::PlayerNotFound, "").dump();
            }
        };
        let last_room_players = self.get_all_players_at_room(player.get_current_room());
        let (room_to_go, room_to_go_id) = {
            let current_player_room_name = player.get_current_room();
            let room_to_go = match self
                .get_neighbor_room_name(current_player_room_name, &direction.to_owned())
            {
                Some(name) => name.clone(),
                None => return generate_json(&leader, command_name, ErrorCode::NoExit, "").dump(),
            };
            let room_id = match self.get_room_by_name(room_to_go.as_str()) {
                Some(room) => room.get_id(),
                None => {
                    warn!("Room not found: {}", room_to_go);
                    return generate_json(&leader, command_name, ErrorCode::NoExit, "").dump();
                }
            };
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

        generate_json(
            &leader,
            command_name,
            ErrorCode::NoError,
            room_repr.as_str(),
        )
        .dump()
    }

    pub fn handle_server_question(&mut self, parsed_json: &JsonValue) -> String {
        let question: &str = parsed_json["question"].as_str().unwrap_or("");
        let data = parsed_json["data"].as_str().unwrap_or("");
        let id = parsed_json["id"].as_str().unwrap_or("");
        match question {
            "ROOM_PLAYERS" => {
                let player_to_check_name = data;
                let player_to_check = self.get_player_from_name(player_to_check_name);
                let player_to_check_current_room = match player_to_check {
                    Some(p) => p.get_current_room(),
                    None => {
                        warn!("Player to check not found: {}", player_to_check_name);
                        return "".to_owned();
                    }
                };
                let players = self.get_all_players_at_room(player_to_check_current_room);
                generate_question_json(question, format!("{:?}", players).as_str(), id).dump()
            }
            _ => {
                error!("unknown question: {}", question);
                "".to_owned()
            }
        }
    }

    pub fn handle_group_command(&mut self, json_object: &JsonValue) -> String {
        let leader = json_object["leader"].as_str().unwrap_or("");
        let grouped_players = json_object["grouped_players"]
            .members()
            .map(|x| x.as_str().unwrap_or("").to_owned())
            .collect::<Vec<String>>();
        let command_name = json_object["command"].as_str().unwrap_or("");
        let leader_id = match self.get_player_id(leader) {
            Some(id) => *id,
            None => {
                warn!("Leader not found: {}", leader);
                return generate_json(leader, command_name, ErrorCode::PlayerNotFound, "")
                    .dump();
            }
        };
        if let Some(already_in_instance) = self.check_player_is_in_instance(leader, leader_id, command_name){
            return already_in_instance.dump();
        }
        let data = json_object["data"].as_str().unwrap_or("");

        match command_name {
            "MOVE" => {
                self.group_command_move(leader.to_owned(), command_name, grouped_players, data)
            }
            "FIGHT_CREATE" => {
                let npc_id = match self.verify_combat_target(leader, command_name, data) {
                    Ok(id) => id,
                    Err(json_response) => return json_response,
                };

                // here call a function with leader id and npc id and grouped players
                self.fight_create_command(leader, npc_id, grouped_players)
            }
            _ => {
                error!("unknown group command: {}", command_name);
                "".to_owned()
            }
        }
    }

    pub fn verify_combat_target(
        &self,
        player_name: &str,
        command_name: &str,
        target_npc: &str,
    ) -> Result<NpcId, String> {
        let player_room = match self.get_player_from_name(player_name) {
            Some(p) => p.get_current_room(),
            None => {
                warn!("Player not found: {}", player_name);
                return Err(generate_json(
                    player_name,
                    command_name,
                    ErrorCode::PlayerNotFound,
                    "",
                )
                .dump());
            }
        };
        let npc_wrapped = self.parse_npc(target_npc, player_room.to_owned());
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
            debug!("received tester response: {}", response);
            let json = json::parse(&response).unwrap_or(json::JsonValue::Null);
            if json.is_null() {
                continue;
            }
            let player = json["player"].as_str().unwrap_or("");
            let npc_id = json["npc_id"].as_u32().unwrap_or(0);
            if let Some(instance) = self.combat_instances.get_mut_instance_for_npc(npc_id)
                && instance.evaluating_players_count > 0
            {
                instance.evaluating_players_count -= 1;
            }
            if let Some(player_id) = self.get_player_id(player).copied() {
                let player_success = json["success"].as_bool().unwrap_or(false);
                if player_success {
                    let instance_player_count = self
                        .get_nb_players_in_player_instance(player_id)
                        .unwrap_or_else(|| {
                            warn!("instance not found for player_id: {}", player_id);
                            1
                        });
                    let npc_combat_start_hp =
                        self.get_npc_combat_start_hp(npc_id).unwrap_or_else(|| {
                            warn!("npc combat start hp not found for npc: {}", npc_id);
                            0
                        });
                    let npc_hp = self.get_npc_hp(npc_id).unwrap_or_else(|| {
                        warn!(
                            "npc {} has no hp and yet he is in a combat instance!",
                            npc_id
                        );
                        0
                    });
                    let dmg =
                        self.calculate_dmg(npc_combat_start_hp, instance_player_count, npc_hp);

                    let players_in_instance = self
                        .get_player_instance_group(player_id)
                        .unwrap_or_else(|| {
                            warn!(
                                "get_player_instance_group returned None for player_id: {}",
                                player_id
                            );
                            vec![]
                        });
                    let event = GameManager::generate_no_player_event_json(
                        &players_in_instance,
                        "FIGHT RESULT",
                        object! { "player_name": player.to_string(), "success": true, "damage_dealt": dmg}.dump().as_str(),
                    );
                    self.add_diff_to_tick(event);
                    self.player_attacks_npc(dmg, player_id, npc_id);
                } else {
                    let players_in_instance = self
                        .get_player_instance_group(player_id)
                        .unwrap_or_else(|| {
                            warn!(
                                "get_player_instance_group returned None for player_id: {}",
                                player_id
                            );
                            vec![]
                        });
                    let npc_dmg = self.generate_npc_dmg();
                    let event = GameManager::generate_no_player_event_json(
                        &players_in_instance,
                        "FIGHT RESULT",
                        object! { "player_name": player.to_string(), "success": false, "damage_dealt": npc_dmg}.dump().as_str(),
                    );
                    self.add_diff_to_tick(event);
                    self.npc_attacks_player(npc_dmg, player_id, npc_id);
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

        let player_name = json_object["player"].as_str().unwrap_or("");
        let player_id = match self.get_player_id(player_name) {
            Some(id) => *id,
            None => {
                warn!("Player not found: {}", player_name);
                return generate_json(player_name, "", ErrorCode::PlayerNotFound, "").dump();
            }
        };
        
        let command_name = json_object["command"].as_str().unwrap_or("");
        let data = json_object["data"].as_str().unwrap_or("");

        info!(
            "received command {} from player {} with args <{}>",
            command_name, player_name, data
        );
        
        if !(command_name == "CONNECT" || command_name == "QUIT" || command_name == "FIGHT_ATTACK") {
            if let Some(already_in_instance) = self.check_player_is_in_instance(player_name, player_id, command_name){
                return already_in_instance.dump();
            }
        }
        match command_name {
            "CONNECT" => {
                if self.get_players_by_names().contains_key(player_name) {
                    self.disconnect_player(player_name.to_owned());
                }
                self.connect_player(player_name.to_owned());
                let player_room = match self.get_player_from_name(player_name) {
                    Some(p) => p.get_current_room(),
                    None => {
                        warn!("Player not found after connect: {}", player_name);
                        return BASE_COMMAND_RESPONSE.to_owned();
                    }
                };
                let mut room_players = self.get_all_players_at_room(player_room);
                let enter_diff = self.generate_event_json(
                    &mut room_players,
                    player_name,
                    "ROOM",
                    "PRESENCE ENTER",
                    true,
                );
                self.add_diff_to_tick(enter_diff);

                BASE_COMMAND_RESPONSE.to_owned()
            }
            "LOOK" => {
                let (
                    player_room_protocol_representation,
                    player_room_name_formatted,
                    player_room_name,
                    player_room_description,
                    player_room_exits,
                    room_players,
                    room_items_str,
                ) = {
                    let player = match self.get_player_from_name(player_name) {
                        Some(p) => p,
                        None => {
                            warn!("Player not found: {}", player_name);
                            return generate_json(
                                player_name,
                                command_name,
                                ErrorCode::PlayerNotFound,
                                "",
                            )
                            .dump();
                        }
                    };
                    let room = match self.get_room_by_name(player.get_current_room()) {
                        Some(r) => r,
                        None => {
                            warn!("Room not found: {}", player.get_current_room());
                            return generate_json(
                                player_name,
                                command_name,
                                ErrorCode::RoomNotFound,
                                "",
                            )
                            .dump();
                        }
                    };
                    (
                        room.get_protocol_representation(),
                        &self.format_room_name(room.get_name()),
                        room.get_name(),
                        room.get_description(),
                        room.get_exits()
                            .iter()
                            .map(|(dir, name)| (dir, self.format_room_name(name)))
                            .collect::<std::collections::HashMap<_, _>>(),
                        self.get_all_players_at_room(player.get_current_room()),
                        self.convert_items_to_string(room.get_inventory()),
                    )
                };

                let room = object! {
                    "room": {
                        "id": player_room_protocol_representation,
                        "name": player_room_name_formatted.as_str(),
                        "description": player_room_description,
                        "exits": JsonValue::from(player_room_exits.clone())
                    },
                    "players": JsonValue::from(room_players),
                    "items": JsonValue::from(room_items_str),
                    "npcs": JsonValue::from(self.get_npcs_in_room_as_protocol_representations(player_room_name))
                };
                generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    room.dump().as_str(),
                )
                .dump()
            }
            "MOVE" => self.group_command_move(player_name.to_owned(), command_name, vec![], data),

            "QUIT" => {

                let player_success = self.get_player_success(player_id);
                // player_success : Option<Option<bool>
                if player_success.is_some() && player_success.unwrap().is_none() {
                    let npc_id = self
                        .combat_instances
                        .get_instance_for_player(player_id)
                        .unwrap()
                        .get_npc_id();
                    self.npc_attacks_player(self.generate_npc_dmg(), player_id, npc_id);
                }

                self.disconnect_player(player_name.to_owned());
                BASE_COMMAND_RESPONSE.to_owned()
            }

            "TALK" => {
                let target_npc = data;
                let player_room = {
                    self.get_player_from_name(player_name)
                        .unwrap()
                        .get_current_room()
                };
                let parsed_repr: Option<(NpcId, String)> =
                    self.parse_npc(target_npc, player_room.to_owned());
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
                if npc_unwrap.get_name() != npc_name || !self.npc_is_in_room(npc_id, player_room) {
                    return generate_json(player_name, command_name, ErrorCode::NpcNotFound, "")
                        .dump();
                }


                let dialog = {
                    let player = match self.get_mut_player_from_name(player_name) {
                        Some(p) => p,
                        None => {
                            warn!("Player not found: {}", player_name);
                            return generate_json(
                                player_name,
                                command_name,
                                ErrorCode::PlayerNotFound,
                                "",
                            )
                            .dump();
                        }
                    };
                    player.talk_with(&npc_unwrap)
                };
                generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    dialog.as_str(),
                )
                .dump()
            }
            // TAKE format : global_id.item_type ( ex: "12.legendary sword")
            "TAKE" => {
                let (player_id, player_room) = {
                    let player = match self.get_player_from_name(player_name) {
                        Some(p) => p,
                        None => {
                            warn!("Player not found: {}", player_name);
                            return generate_json(
                                player_name,
                                command_name,
                                ErrorCode::PlayerNotFound,
                                "",
                            )
                            .dump();
                        }
                    };
                    (player.get_id(), player.get_current_room().to_owned())
                };
                let Some(room) = self.get_room_by_name(player_room.as_str()) else {
                        warn!("Room not found: {}", player_room);
                        return generate_json(player_name, command_name, ErrorCode::RoomNotFound, "")
                            .dump();
                    };
                let item = data;
                let room_name = room.get_name().to_owned();
                let parsed_item: Option<(ItemId, String)> =
                    self.parse_item(item, &room);
                if parsed_item.is_none() {
                    return generate_json(player_name, command_name, ErrorCode::ItemNotFound, "")
                        .dump();
                }
                let (item_id, item_name) = parsed_item.unwrap();

                if !self.item_exists_with_name(item_id, item_name.as_str()) || !room.contains_item(item_id){
                    return generate_json(
                        player_name,
                        command_name,
                        ErrorCode::ItemNotFound,
                        "",
                    )
                    .dump();
                }

                self.remove_item_from_room(room_name.as_str(), item_id);
                self.add_item_to_player(player_id, item_id);
                self.reset_dropped_at_for_item(item_id);

                let mut players_to_send = self.get_all_players_at_room(player_room.as_str());
                let events_json =
                    self.generate_event_json(&mut players_to_send, player_name, "TAKE", item, true);
                self.add_diff_to_tick(events_json);

                generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    self.get_item_name_from_id(item_id).to_string().as_str(),
                )
                .dump()
            }
            "DROP" => {
                let player = match self.get_player_from_name(player_name) {
                    Some(p) => p,
                    None => {
                        warn!("Player not found: {}", player_name);
                        return generate_json(
                            player_name,
                            command_name,
                            ErrorCode::PlayerNotFound,
                            "",
                        )
                        .dump();
                    }
                };

                let player_id = player.get_id();
                let item = data;
                let room_name = player.get_current_room().to_owned();
                let item_tuple: Option<(ItemId, String)> =
                    self.parse_item_from_player(item, &player);
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

                generate_json(player_name, command_name, ErrorCode::NoError, item).dump()
            }
            "INVENTORY" => {
                let inventory = self.get_player_inventory_as_string(player_name);
                generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    inventory.as_str(),
                )
                .dump()
            }
            "FIGHT_CREATE" => {
                let npc_id = match self.verify_combat_target(player_name, command_name, data) {
                    Ok(id) => id,
                    Err(json_response) => return json_response,
                };

                // here call a function with leader id and npc id and grouped players
                self.fight_create_command(player_name, npc_id, Vec::new())
            }
            "FIGHT_ATTACK" => {
                let file_and_npc_id = {
                    let player_id = match self.get_player_id(player_name) {
                        Some(id) => *id,
                        None => {
                            warn!("Player not found: {}", player_name);
                            return generate_json(
                                player_name,
                                command_name,
                                ErrorCode::PlayerNotFound,
                                "",
                            )
                            .dump();
                        }
                    };
                    if let Some(instance) = self.combat_instances.get_instance_for_player(player_id)
                    {
                        let file_name = instance.get_assigned_file_name().to_string();
                        let npc_id = instance.get_npc_id();
                        if self.check_action_already_taken(player_id, npc_id) {
                            return generate_json(
                                player_name,
                                command_name,
                                ErrorCode::ActionAlreadyTaken,
                                "",
                            )
                            .dump();
                        }
                        Some((file_name, npc_id))
                    } else {
                        None
                    }
                };
                if file_and_npc_id.is_none() {
                    return generate_json(
                        player_name,
                        command_name,
                        ErrorCode::PlayerNotInCombat,
                        "",
                    )
                    .dump();
                }
                let (file_name, npc_id) = file_and_npc_id.unwrap();

                let sent_code = data;

                /*check if the code is correct*/
                self.test_code(&file_name, sent_code, player_name, npc_id);
                generate_json(player_name, command_name, ErrorCode::NoError, "Processing").dump()
            }
            "ATTACK" => {
                let npc_id = match self.verify_combat_target(player_name, command_name, data) {
                    Ok(id) => id,
                    Err(json_response) => return json_response,
                };
                let player_id = match self.get_player_id(player_name) {
                    Some(id) => *id,
                    None => {
                        warn!("Player not found: {}", player_name);
                        return generate_json(
                            player_name,
                            command_name,
                            ErrorCode::PlayerNotFound,
                            "",
                        )
                        .dump();
                    }
                };

                let combat_result = self.player_attacks_npc(1, player_id, npc_id);

                generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    combat_result.as_str(),
                )
                .dump()
            }
            "STATUS" => {
                let player_status = self.get_player_status_as_string(player_name);
                generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    player_status.as_str(),
                )
                .dump()
            }
            "QUEST" => {
                let target_npc = data;
                let Some(player) = self.get_player_from_name(player_name) else {
                    return generate_json(
                                player_name,
                                command_name,
                                ErrorCode::PlayerNotFound,
                                "",
                            )
                            .dump();
                };
                let player_room = player.get_current_room();
                // if player
                let parsed_repr: Option<(NpcId, String)> =
                    self.parse_npc(target_npc, player_room.to_owned());
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
                    let player_id = match self.get_player_id(player_name) {
                        Some(id) => *id,
                        None => {
                            warn!("Player not found: {}", player_name);
                            return generate_json(
                                player_name,
                                command_name,
                                ErrorCode::PlayerNotFound,
                                "",
                            )
                            .dump();
                        }
                    };

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
                                "name" => quest.get_name().to_string(),
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

                        let player_id = match self.get_player_id(player_name) {
                            Some(id) => *id,
                            None => {
                                warn!("Player not found: {}", player_name);
                                return generate_json(
                                    player_name,
                                    command_name,
                                    ErrorCode::PlayerNotFound,
                                    "",
                                )
                                .dump();
                            }
                        };
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
                generate_json(player_name, command_name, ErrorCode::NoQuestAvailable, "").dump()
            }
            "QUESTS" => {
                let player_id = match self.get_player_id(player_name) {
                    Some(id) => *id,
                    None => {
                        warn!("Player not found: {}", player_name);
                        return generate_json(
                            player_name,
                            command_name,
                            ErrorCode::PlayerNotFound,
                            "",
                        )
                        .dump();
                    }
                };

                let quests = self
                    .quest_instances
                    .iter()
                    .filter(|q| q.get_player() == player_id)
                    .filter_map(|q| {
                        let quest = self.get_quest(&q.get_quest_name());
                        if let Some(quest) = quest {
                            Some(json::object! {
                            "name" => quest.get_name().to_string(),
                            "description" => quest.get_description(),
                            "reward" => quest.get_json_loots(),
                            "status" => q.get_state().to_str() })
                        } else {
                            warn!("quest not found: {}", q.get_quest_name());
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let quests_json: JsonValue = JsonValue::Array(quests);

                generate_json(
                    player_name,
                    command_name,
                    ErrorCode::NoError,
                    quests_json.dump().as_str(),
                )
                .dump()
            }
            _ => {
                println!("Unknown command: {}", command_name);
                "".to_owned()
            }
        }
    }
}
