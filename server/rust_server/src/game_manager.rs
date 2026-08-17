use crate::combat_instances::CombatInstanceManager;
use crate::constantes::{
    CODE_NL_SEP, CODE_SP_SEP, Direction, MAX_TIME_FOR_COMBAT, NPC_DMG, NPC_RESPAWN_TIME,
    PLAYER_ROOM_SPAWN, TEST_FILES_DIR,
};
use crate::inventory::Inventory;
use crate::items::{Item, ItemId};
use crate::npc::{Npc, NpcId};
use crate::parser::Parser;
use crate::player::{Player, PlayerCount, PlayerId};
use crate::quests::{Quest, QuestInstance, QuestState, Questid};
use crate::room::{Room, RoomId, RoomName};
use crate::save::{Save, ServerSave};
use crate::tester::test;
use json::{JsonValue, object};
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::sync::mpsc;
use std::time::Instant;
use tracing::warn;
use tracing::{debug, error, info};

pub struct GameManager {
    players: HashMap<PlayerId, Player>,
    players_by_name: HashMap<String, PlayerId>,
    next_player_id: PlayerCount,
    next_item_id: ItemId,
    pub all_items: HashMap<ItemId, Item>,
    pub all_rooms: HashMap<RoomId, Room>,
    pub all_npcs: HashMap<NpcId, Npc>,
    pub all_quests: HashMap<Questid, Quest>,
    pub quest_instances: Vec<QuestInstance>,
    pub combat_instances: CombatInstanceManager,
    mpsc_receiver: mpsc::Receiver<String>,
    pub tester_receiver: mpsc::Receiver<String>,
    pub tester_sender: mpsc::Sender<String>,
    writer_stream: TcpStream,
    tick_diff: HashMap<String, JsonValue>,
}

impl GameManager {
    pub fn new(
        mpsc_receiver: mpsc::Receiver<String>,
        tester_receiver: mpsc::Receiver<String>,
        tester_sender: mpsc::Sender<String>,
        writer_stream: TcpStream,
        parser: Parser,
    ) -> Self {
        let mut manager = Self {
            players: HashMap::new(),
            players_by_name: HashMap::new(),
            next_player_id: 0,
            next_item_id: 0,
            all_items: parser.get_items().clone(),
            all_rooms: parser.get_rooms().clone(),
            all_npcs: parser.get_npcs().clone(),
            all_quests: parser.get_quests().clone(),
            combat_instances: CombatInstanceManager::new(),
            quest_instances: Vec::new(),
            mpsc_receiver,
            tester_receiver,
            tester_sender,
            writer_stream,
            tick_diff: HashMap::new(),
        };

        manager.restore_server_state();
        return manager;
    }

    pub fn get_players(&self) -> &HashMap<PlayerId, Player> {
        return &self.players;
    }

    fn save_player(&mut self, player_id: PlayerId) {
        if let Some(player) = self.players.get(&player_id) {
            let inventory = player.get_inventory().clone();
            let player_quests = self
                .quest_instances
                .iter()
                .filter(|q| q.get_player() == player_id)
                .map(|q| (q.get_quest_name(), q.get_state()))
                .collect();
            let save_data = Save {
                name: player.get_name().to_owned(),
                id: player.get_id(),
                hp: player.get_hp(),
                max_hp: player.get_max_hp(),
                inventory,
                current_room: player.get_current_room().to_owned(),
                dialogs_index: std::collections::HashMap::new(),
                quests: player_quests,
            };
            if let Err(e) =
                confy::store_path(format!("saves/{}.toml", player.get_name()), save_data)
            {
                error!("Failed to save player: {}", e);
            }
        } else {
            warn!(
                "Player not found: {} while saving the game progression",
                player_id
            );
        }
    }

    pub fn get_player(&self, player_id: PlayerId) -> Option<&Player> {
        self.players.get(&player_id)
    }

    pub fn get_mut_player(&mut self, player_id: PlayerId) -> Option<&mut Player> {
        self.players.get_mut(&player_id)
    }

    pub fn get_player_id(&self, player_name: &str) -> Option<&PlayerId> {
        self.players_by_name.get(player_name)
    }
    pub fn get_players_by_names(&self) -> &HashMap<String, PlayerId> {
        return &self.players_by_name;
    }

    pub fn get_player_from_name(&self, player_name: &str) -> Option<&Player> {
        let player_id = self.get_player_id(player_name);
        match player_id {
            Some(player_id) => self.get_player(*player_id),
            _none => None,
        }
    }

    pub fn get_mut_player_from_name(&mut self, player_name: &str) -> Option<&mut Player> {
        let player_id = self.get_player_id(player_name).copied();
        match player_id {
            Some(player_id) => self.players.get_mut(&player_id),
            _none => None,
        }
    }

    pub fn save_server_state(&mut self) {
        let player_ids: Vec<PlayerId> = self.players.keys().copied().collect();
        for id in player_ids {
            self.save_player(id);
        }

        let mut rooms_inventory = HashMap::new();
        for (room_id, room) in &self.all_rooms {
            rooms_inventory.insert(room_id.to_string(), room.get_inventory().clone());
        }

        let server_save = crate::save::ServerSave {
            next_player_id: self.next_player_id,
            next_item_id: self.next_item_id,
            rooms_inventory,
        };

        if let Err(e) = confy::store_path("saves/server_state.toml", server_save) {
            tracing::error!("Failed to save server state: {}", e);
        }
    }

    fn restore_server_state(&mut self) {
        let path = "saves/server_state.toml";
        if !std::path::Path::new(path).exists() {
            return;
        }
        let Ok(server_save) = confy::load_path::<ServerSave>(path) else {
            self.set_default_ids();
            return;
        };

        if server_save.next_player_id > 0 || server_save.next_item_id > 1 {
            self.next_player_id = server_save.next_player_id;
            self.next_item_id = server_save.next_item_id;
            for (room_id_str, mut inventory) in server_save.rooms_inventory {
                let items_to_check: Vec<ItemId> = inventory.get_items().iter().cloned().collect();
                for item_id in items_to_check {
                    if !self.item_exists(item_id) {
                        tracing::warn!(
                            "Removing invalid item {} from room {}",
                            item_id,
                            room_id_str
                        );
                        inventory.remove_item(item_id);
                    }
                }

                if let Ok(room_id) = room_id_str.parse::<u32>() {
                    if let Some(room) = self.all_rooms.get_mut(&room_id) {
                        room.set_inventory(inventory);
                    }
                } else {
                    self.set_default_ids();
                }
            }
        } else {
            self.set_default_ids();
        }
    }

    pub fn set_default_ids(&mut self) {
        self.next_player_id = 0;
        self.next_item_id = self.all_items.len() as ItemId;
    }

    pub fn get_all_items(&mut self) -> &mut HashMap<ItemId, Item> {
        return &mut self.all_items;
    }

    pub fn get_all_rooms(&mut self) -> &mut HashMap<RoomId, Room> {
        return &mut self.all_rooms;
    }

    pub fn get_quest(&self, id: &Questid) -> Option<&Quest> {
        self.all_quests.get(id)
    }

    pub fn get_all_quests(&mut self) -> &mut HashMap<Questid, Quest> {
        return &mut self.all_quests;
    }

    fn try_restore_player_save(&mut self, name: &str) -> Option<Player> {
        let path = format!("saves/{}.toml", name);

        if !std::path::Path::new(&path).exists() {
            return None;
        }
        let Ok(mut save_data) = confy::load_path::<Save>(&path) else {
            return None;
        };

        if save_data.name != name {
            return None;
        }

        if !self.room_exists(&save_data.current_room) {
            return None;
        }

        // Filter out nonexistent items from player's inventory
        let items_to_check: Vec<ItemId> = save_data.inventory.get_items().iter().cloned().collect();
        for item_id in items_to_check {
            if !self.item_exists(item_id) {
                warn!(
                    "Removing invalid item {} from player {}",
                    item_id, save_data.name
                );
                save_data.inventory.remove_item(item_id);
            }
        }

        let player_id = save_data.id;
        for (quest_id, state) in save_data.quests.iter() {
            let quest_instance = QuestInstance::new(player_id, quest_id.clone(), state.clone());
            self.quest_instances.push(quest_instance);
        }

        return Some(Player::from_save(save_data));
    }

    fn add_player_to_game(&mut self, player: Player) {
        let player_name = player.get_name().to_owned();
        let player_id = player.get_id();
        self.players.insert(player_id, player);
        self.players_by_name.insert(player_name, player_id);
    }

    fn create_new_player(&mut self, name: String) {
        let player_id = self.next_player_id;
        let player = Player::new(name.clone(), player_id);

        self.add_player_to_game(player);
        self.next_player_id += 1;
    }

    pub fn connect_player(&mut self, name: String) {
        match self.try_restore_player_save(&name) {
            Some(player) => self.add_player_to_game(player),
            _none => self.create_new_player(name),
        }
    }

    pub fn disconnect_player(&mut self, name: String) {
        let player_id = match self.players_by_name.get(&name) {
            Some(&id) => id,
            _none => {
                error!("disconnect player: player not found");
                return;
            }
        };
        self.save_player(player_id);
        self.players.remove(&player_id);
        self.players_by_name.remove(&name);
        self.quest_instances.retain(|q| q.get_player() != player_id);
    }

    pub fn get_nb_players(&self) -> usize {
        return self.players.len();
    }

    pub fn get_item_name(&self, item_id: &ItemId) -> String {
        self.all_items.get(item_id).unwrap().get_name().to_owned()
    }

    pub fn remove_item_from_player(&mut self, player_id: PlayerId, item_id: ItemId) {
        let player = self.players.get_mut(&player_id).unwrap();
        player.remove_item(item_id);
    }

    pub fn add_item_to_player(&mut self, player_id: PlayerId, item_id: ItemId) {
        let player = self.players.get_mut(&player_id).unwrap();
        player.add_item(item_id);
    }

    pub fn get_player_inventory_as_string(&self, player_name: &str) -> String {
        let player = self.get_player_from_name(player_name).unwrap();

        let items: Vec<String> = player
            .get_items()
            .iter()
            .map(|item_id| format!("{}.{}", item_id, self.get_item_name(item_id)))
            .collect();

        return format!("{:?}", items);
    }

    pub fn get_room_by_name(&self, room_name: &str) -> Option<&Room> {
        self.all_rooms
            .values()
            .find(|room| room.get_name() == room_name)
    }

    pub fn get_mut_room_by_name(&mut self, room_name: &str) -> Option<&mut Room> {
        self.all_rooms
            .values_mut()
            .find(|room| room.get_name() == room_name)
    }

    pub fn get_room(&self, room_id: RoomId) -> Option<&Room> {
        self.all_rooms.get(&room_id)
    }

    pub fn remove_item_from_room(&mut self, room_name: &str, item_id: ItemId) {
        let room = self.get_mut_room_by_name(room_name).unwrap();
        room.remove_item(item_id);
    }

    pub fn add_item_to_room(&mut self, room_name: &str, item_id: ItemId) {
        let room = self.get_mut_room_by_name(room_name).unwrap();
        room.add_item(item_id);
    }

    pub fn send_msg_to_client(&mut self, msg: String) -> std::io::Result<()> {
        self.writer_stream.write_all((msg + "\n").as_bytes())?;
        Ok(())
    }

    pub fn receive_data_timeout(
        &mut self,
        duration: std::time::Duration,
    ) -> Result<String, std::sync::mpsc::RecvTimeoutError> {
        return self.mpsc_receiver.recv_timeout(duration);
    }

    pub fn item_exists(&self, item_id: ItemId) -> bool {
        return self.all_items.contains_key(&item_id);
    }

    pub fn room_exists(&self, room_name: &str) -> bool {
        self.all_rooms
            .values()
            .any(|room| room.get_name() == room_name)
    }

    pub fn player_exists(&self, player_name: &str) -> bool {
        self.players_by_name.contains_key(player_name)
    }

    pub fn get_only_item_with_name(&self, item_name: &str) -> Option<ItemId> {
        let mut count = 0;
        let mut item_id: Option<ItemId> = None;
        for (_, item) in self.all_items.iter() {
            if item.get_name() == item_name {
                count += 1;
                item_id = Some(item.get_id());
            }
        }
        if count == 1 { item_id } else { None }
    }

    pub fn item_exists_with_name(&self, item_id: ItemId, item_name: &str) -> bool {
        let item = self.all_items.get(&item_id);
        item.is_some_and(|item| item.get_name() == item_name)
    }

    pub fn get_neighbor_room_name(
        &self,
        room_name: &str,
        direction: &Direction,
    ) -> Option<&RoomName> {
        let room = self.get_room_by_name(room_name);
        match room {
            Some(room) => room.get_neighbor_room_name(direction),
            _none => None,
        }
    }

    pub fn get_tick_diff(&self) -> &HashMap<String, JsonValue> {
        &self.tick_diff
    }

    pub fn add_diff_to_tick(&mut self, diff: JsonValue) {
        let players = diff["players"].members();
        let mut filtered = diff.clone();
        filtered.remove("players");
        for player in players {
            let key = player.as_str().unwrap().to_owned();
            let entry = self.tick_diff.entry(key).or_insert(JsonValue::new_array());

            entry.push(filtered.clone()).unwrap();
        }
    }
    pub fn clear_diff(&mut self) {
        self.tick_diff.clear();
    }

    pub fn get_all_players_at_room(&self, room_name: &str) -> Vec<String> {
        let mut players: Vec<String> = Vec::new();
        for player in self.players.values() {
            if player.get_current_room() == room_name {
                players.push(player.get_name().to_owned());
            }
        }
        return players;
    }

    pub fn npc_is_in_room(&self, npc_id: NpcId, room_name: &str) -> bool {
        self.get_npc(npc_id)
            .map_or(false, |npc| npc.get_spawn_room() == room_name)
    }

    pub fn move_player_to_room(&mut self, player_name: &str, room_name: &str) {
        let player = self.get_mut_player_from_name(player_name).unwrap();
        player.move_to_room(&room_name.to_owned());
    }

    pub fn get_npcs_in_room_as_protocol_representations(&self, room_name: &str) -> Vec<String> {
        self.all_npcs
            .values()
            .filter(|&npc| npc.get_spawn_room() == room_name && npc.get_death().is_none())
            .map(|npc| npc.get_protocol_representation())
            .collect()
    }

    pub fn punish_inactive_players_in_combat(&mut self) {
        let mut players_to_punish: Vec<(PlayerId, NpcId)> = Vec::new();
        for (npc_id, instance) in self.combat_instances.instances.iter() {
            if instance.combat_start_time.elapsed() > MAX_TIME_FOR_COMBAT
                && !instance.is_evaluating_response
            {
                for (player_id, success) in instance.players_success.iter() {
                    if success.is_none() {
                        players_to_punish.push((*player_id, *npc_id));
                    }
                }
            }
        }
        for (player_id, npc_id) in players_to_punish {
            self.npc_attacks_player(NPC_DMG, player_id, npc_id);
        }
    }

    pub fn revive_dead_npcs(&mut self) {
        let mut npcs_to_revive = Vec::new();
        for (npc_id, ncp) in self.all_npcs.iter() {
            if let Some(death) = ncp.get_death() {
                if death.elapsed() > NPC_RESPAWN_TIME {
                    npcs_to_revive.push(*npc_id);
                }
            }
        }

        for npc_id in npcs_to_revive {
            let (room_name, ncp_rep) = {
                let ncp = self.all_npcs.get_mut(&npc_id).unwrap();
                ncp.revive();
                (
                    ncp.get_spawn_room().to_owned(),
                    ncp.get_protocol_representation(),
                )
            };
            let players_to_send_event = self.get_all_players_at_room(&room_name);
            let data = format!("type=NPC id={}", ncp_rep);
            let event =
                GameManager::generate_no_player_event_json(&players_to_send_event, "SPAWN", &data);
            self.add_diff_to_tick(event);
        }
    }

    pub fn get_npc_combat_start_hp(&self, npc_id: NpcId) -> Option<u32> {
        let instance = self.combat_instances.get_instance_for_npc(npc_id);
        instance.map(|i| i.get_npc_combat_start_hp())
    }

    pub fn get_npc(&self, npc_id: NpcId) -> Option<&Npc> {
        self.all_npcs.get(&npc_id)
    }

    pub fn parse_npc(&self, npc_rep: &str, room_needed: RoomName) -> Option<(NpcId, String)> {
        if let Some(npc_wrapped) = Npc::parse_protocol_representation(npc_rep) {
            let npc = self.get_npc(npc_wrapped.0)?;
            return Some((npc.get_id(), npc.get_name()));
        }

        self.all_npcs
            .iter()
            .find(|(_, npc)| npc.get_spawn_room() == room_needed && npc.get_name() == npc_rep)
            .map(|(npc_id, npc)| (npc_id.clone(), npc.get_name().clone()))
    }

    pub fn parse_item(&self, item_rep: &str, room_needed: RoomName) -> Option<(ItemId, String)> {
        if let Some(item) = Item::parse_item(item_rep) {
            return Some(item);
        }
        let room_items = self
            .get_room_by_name(room_needed.as_str())
            .unwrap()
            .get_inventory()
            .get_items()
            .clone();
        room_items
            .iter()
            .find(|item_id| self.get_item_name(item_id) == item_rep)
            .map(|item_id| (item_id.clone(), self.get_item_name(item_id)))
    }

    pub fn convert_items_to_string(&self, inventory: &Inventory) -> Vec<String> {
        inventory
            .get_items()
            .iter()
            .map(|item_id| format!("{}.{}", item_id, self.get_item_name(item_id)))
            .collect()
    }

    pub fn reset_dropped_at_for_item(&mut self, item_id: ItemId) {
        if let Some(item) = self.all_items.get_mut(&item_id) {
            item.stop_dropped_at();
        }
    }

    pub fn start_dropped_at_for_item(&mut self, item_id: ItemId) {
        if let Some(item) = self.all_items.get_mut(&item_id) {
            item.set_dropped_at(Instant::now());
        }
    }

    pub fn get_player_status_as_string(&self, player_name: &str) -> String {
        let player = self.get_player_from_name(player_name).unwrap();
        let hp = player.get_hp();
        let max_hp = player.get_max_hp();
        let percentage_hp = hp as f64 / max_hp as f64 * 100.0;
        let status = if percentage_hp >= 80.0 {
            "healthy"
        } else if percentage_hp >= 50.0 {
            "normal"
        } else {
            "critical"
        };

        format!(
            "{{\"hp\":{}, \"max_hp\":{}, \"status\":\"{}\"}}",
            player.get_hp(),
            player.get_max_hp(),
            status
        )
    }

    pub fn get_mut_npc(&mut self, npc_id: NpcId) -> Option<&mut Npc> {
        self.all_npcs.get_mut(&npc_id)
    }

    pub fn get_npc_type(&self, npc_id: NpcId) -> u8 {
        self.all_npcs
            .get(&npc_id)
            .map(|npc| npc.get_npc_type())
            .unwrap()
    }

    pub fn get_npc_max_hp(&self, npc_id: NpcId) -> Option<u32> {
        self.all_npcs.get(&npc_id).and_then(|npc| npc.get_max_hp())
    }

    pub fn get_npc_hp(&self, npc_id: NpcId) -> Option<u32> {
        self.all_npcs.get(&npc_id).and_then(|npc| npc.get_hp())
    }

    pub fn kill_npc(&mut self, npc_id: NpcId) {
        if let Some(npc) = self.get_mut_npc(npc_id) {
            npc.die();
        }
        if let Some(instance) = self.combat_instances.get_mut_instance_for_npc(npc_id) {
            instance.force_finish();
        }
    }

    pub fn kill_player(&mut self, player_id: PlayerId) {
        let (mut players_to_send_death_info, player_name) = {
            let player = self.get_player(player_id).unwrap();
            let mut players = self.get_all_players_at_room(player.get_current_room());
            players.extend(self.get_all_players_at_room(PLAYER_ROOM_SPAWN));

            (players, player.get_name().to_owned())
        };
        let path = format!("saves/{}.toml", player_name);
        let _ = std::fs::remove_file(path);
        // ignore error because the save may not exist yet
        // (in which case we do not need to delete the save)
        info!("deleted player {} save", player_name);
        self.get_mut_player(player_id).unwrap().reset();
        let event = self.generate_event_json(
            &mut players_to_send_death_info,
            player_name.as_str(),
            "DEATH",
            format!("respawn_room_id={}", PLAYER_ROOM_SPAWN).as_str(),
            false,
        );
        self.add_diff_to_tick(event);
    }

    pub fn get_player_instance_group(&self, player_id: PlayerId) -> Option<Vec<String>> {
        if let Some(instance) = self.combat_instances.get_instance_for_player(player_id) {
            let mut vec = Vec::new();
            for player_id in instance.get_grouped_players() {
                if let Some(player) = self.get_player(*player_id) {
                    vec.push(player.get_name().to_owned());
                }
            }
            if let Some(leader) = self.get_player(instance.get_leader()) {
                vec.push(leader.get_name().to_owned());
            }
            return Some(vec);
        }
        None
    }

    pub fn check_action_already_taken(&self, player_id: PlayerId, _npc_id: NpcId) -> bool {
        if let Some(instance) = self.combat_instances.get_instance_for_player(player_id) {
            if let Some(_player) = instance.get_player_success(player_id) {
                if let Some(_success) = _player {
                    return true;
                }
            }
        }
        false
    }
    pub fn npc_attacks_player(
        &mut self,
        damage: u32,
        player_id: NpcId,
        npc_id: PlayerId,
    ) -> String {
        let npc = self.get_mut_npc(npc_id).unwrap();

        debug!(
            "npc: {}, npc_id_received: {}",
            npc.get_protocol_representation(),
            npc_id
        );
        let npc_hp = npc.get_hp().unwrap();
        let mut dealt_damage = damage;
        let player = self.get_mut_player(player_id).unwrap();
        let player_name = player.get_name().to_owned();
        let player_hp = player.get_hp();
        let new_player_hp = if player_hp > damage {
            player_hp - damage
        } else {
            dealt_damage = player_hp;
            0
        };

        // TODO: event dans defend qui dis combien de degats le joueur defend ( envoyé aux membres du groupe)
        // pareil dans Attack
        // faire que quand un joueur quitte, Defend soit lancé automatiquement et il est marqué comme finished dans l'instance
        // sur le point au dessus manque plus que de mettre des degats au joueur quand il quitte

        let status = if player_hp > 0 { "combat" } else { "death" };
        player.set_hp(new_player_hp);

        //does nothing if no the player is not in a combat instance
        self.set_success_for_player(player_id, false);
        let mut players_to_send_event = self
            .combat_instances
            .get_all_players_in_combat(npc_id)
            .iter()
            .map(|player_id| self.get_player(*player_id).unwrap().get_name().to_owned())
            .collect::<Vec<String>>();
        let event = self.generate_event_json(
            &mut players_to_send_event,
            player_name.as_str(),
            "DEFEND",
            dealt_damage.to_string().as_str(),
            false,
        );
        self.add_diff_to_tick(event);

        if new_player_hp == 0 {
            self.kill_player(player_id);
        }
        return format!(
            "{{\"attacker_hp\":{}, \"target_hp\":{}, \"damage\":{}, \"status\":\"{}\"}}",
            npc_hp, new_player_hp, dealt_damage, status
        );
    }

    pub fn get_player_success(&self, player_id: PlayerId) -> Option<Option<bool>> {
        if let Some(instance) = self.combat_instances.get_instance_for_player(player_id) {
            return instance.get_player_success(player_id);
        }
        None
    }
    pub fn set_success_for_player(&mut self, player_id: PlayerId, success: bool) {
        if let Some(instance) = self.combat_instances.get_mut_instance_for_player(player_id) {
            instance.set_player_success(player_id, success);
        }
    }

    pub fn player_attacks_npc(
        &mut self,
        damage: u32,
        player_id: PlayerId,
        npc_id: NpcId,
    ) -> String {
        let player = self.get_player(player_id).unwrap();
        let npc_room = self.get_npc(npc_id).unwrap().get_spawn_room().to_owned();
        let mut players_in_room = self.get_all_players_at_room(npc_room.as_str()).clone();
        let player_name = player.get_name().to_owned();
        let player_hp = player.get_hp();
        let npc = self.get_mut_npc(npc_id).unwrap();
        let npc_repr = npc.get_protocol_representation();
        let hp = npc.get_hp().unwrap();
        let mut dealt_damage = damage;
        let new_npc_hp = if hp > damage {
            hp - damage
        } else {
            dealt_damage = hp;
            0
        };

        let status = if new_npc_hp > 0 { "combat" } else { "victory" };
        npc.set_hp(Some(new_npc_hp));

        //does nothing if no the player is not in a combat instance
        self.set_success_for_player(player_id, true);
        // if let Some(mut players_to_send_event) = self.get_player_instance_group(player_id) {
        //     let event = self.generate_event_json(
        //         &mut players_to_send_event,
        //         &player_name,
        //         "ATTACK",
        //         dealt_damage.to_string().as_str(),
        //         true,
        //     );
        //     self.add_diff_to_tick(event);
        // }
        debug!("npc hp:{}", new_npc_hp);
        if new_npc_hp == 0 {
            debug!("killed npc");
            self.kill_npc(npc_id);
            let event = self.generate_event_json(
                &mut players_in_room,
                &player_name,
                "KILL",
                npc_repr.as_str(),
                false,
            );
            self.add_diff_to_tick(event);
        }
        return format!(
            "{{\"attacker_hp\":{}, \"target_hp\":{}, \"damage\":{}, \"status\":\"{}\"}}",
            player_hp, new_npc_hp, dealt_damage, status
        );
    }

    pub fn player_has_quest(&self, player_id: PlayerId, quest_id: Questid) -> bool {
        self.quest_instances.iter().any(|quest_instance| {
            quest_instance.get_player() == player_id
                && quest_instance.get_quest_name() == quest_id
                && quest_instance.get_state() == QuestState::InProgress
        })
    }

    pub fn player_has_item(&self, player_id: PlayerId, item_id: ItemId) -> bool {
        return self.get_player(player_id).unwrap().has_item(item_id);
    }

    pub fn get_mut_item(&mut self, item_id: ItemId) -> &mut Item {
        self.all_items.get_mut(&item_id).unwrap()
    }

    pub fn get_random_test_file_name(&self) -> String {
        let mut all_files = Vec::new();
        match std::fs::read_dir(TEST_FILES_DIR) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(name) = path.file_name() {
                                    all_files.push(name.to_str().unwrap().to_owned());
                                }
                            }
                        }
                        Err(e) => {
                            warn!("error {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read test files directory: {}", e);
            }
        }

        if all_files.is_empty() {
            return String::new();
        }

        use rand::RngExt;
        let mut rng = rand::rng();
        let idx = rng.random_range(0..all_files.len());
        all_files[idx].clone()
    }

    pub fn get_room_id_from_name(&self, room_name: &str) -> RoomId {
        self.all_rooms
            .values()
            .find(|room| room.get_name() == room_name)
            .map(|room| room.get_id())
            .unwrap_or(2 as RoomId)
    }
    pub fn get_finished_instances_players(&mut self) -> Vec<Vec<PlayerId>> {
        let mut vec: Vec<Vec<PlayerId>> = Vec::new();
        let mut players: Vec<PlayerId> = Vec::new();
        for (_npc_id, instance) in self.combat_instances.instances.iter() {
            if instance.all_players_finished() {
                players.extend(instance.get_grouped_players());
                players.push(instance.get_leader());
                vec.push(players.clone());
                players.clear();
            }
        }
        vec
    }
    pub fn remove_finished_combat_instances(&mut self) {
        let finished_instances_players = self.get_finished_instances_players();
        if finished_instances_players.is_empty() {
            return;
        }
        debug!("finished players: {:?}", finished_instances_players);
        for grouped_players in finished_instances_players {
            if !grouped_players.is_empty() {
                let mut grouped_players_strings: Vec<String> = Vec::new();
                for player in grouped_players {
                    if let Some(player) = self.get_player(player) {
                        grouped_players_strings.push(player.get_name().to_owned());
                    }
                }
                let event = GameManager::generate_no_player_event_json(
                    &grouped_players_strings,
                    "FIGHT END",
                    "",
                );
                self.add_diff_to_tick(event);
            }
        }
        self.combat_instances.remove_finished_instances();
    }

    pub fn test_code(&mut self, file_name: &str, sent_code: &str, player: &str, npc_id: NpcId) {
        let sender = self.tester_sender.clone();
        let mut response = object! {"player": player, "npc_id": npc_id, "success": false};
        let file_name_owned = file_name.to_owned();
        let sent_code_owned = sent_code
            .to_owned()
            .replace(CODE_NL_SEP, "\n")
            .replace(CODE_SP_SEP, " ");
        // debug!("code sent to ldecavel: {}", sent_code_owned);

        let instance = self
            .combat_instances
            .get_mut_instance_for_npc(npc_id)
            .unwrap();
        instance.is_evaluating_response = true;
        std::thread::spawn(move || {
            let result = test(&file_name_owned, &sent_code_owned);
            response["success"] = result.into();
            let _ = sender.send(response.dump());
        });
    }
    pub fn get_nb_players_in_player_instance(&self, player_id: PlayerId) -> Option<u32> {
        self.combat_instances
            .instances
            .values()
            .find_map(|instance| {
                if instance.get_leader() == player_id
                    || instance.get_grouped_players().contains(&player_id)
                {
                    Some(1 + instance.get_grouped_players().len() as u32)
                } else {
                    None
                }
            })
    }

    pub fn is_npc_in_combat(&self, npc_id: NpcId) -> bool {
        self.combat_instances.get_instance_for_npc(npc_id).is_some()
    }
}
