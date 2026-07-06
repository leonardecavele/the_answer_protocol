use crate::constantes::{Direction, LOST_ITEM_SPAWN};
use crate::inventory::Inventory;
use crate::items::{Item, ItemId};
use crate::npc::{Npc, NpcId};
use crate::parser::Parser;
use crate::player::{Player, PlayerCount, PlayerId};
use crate::quests::{Quest, QuestInstance, QuestState, Questid};
use crate::room::{Room, RoomId, RoomName};
use crate::save::{Save, ServerSave};
use json::JsonValue;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use std::time::Instant;
use tracing::error;
use tracing::warn;

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
    mpsc_receiver: mpsc::Receiver<String>,
    writer_stream: TcpStream,
    tick_diff: HashMap<String, JsonValue>,
}

impl GameManager {
    pub fn new(
        mpsc_receiver: mpsc::Receiver<String>,
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
            quest_instances: Vec::new(),
            mpsc_receiver,
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
            let save_data = crate::save::Save {
                name: player.get_name().to_string(),
                id: player.get_id(),
                hp: player.get_hp(),
                max_hp: player.get_max_hp(),
                inventory,
                current_room: player.get_current_room().to_string(),
                dialogs_index: std::collections::HashMap::new(),
            };
            if let Err(e) = confy::store_path(format!("saves/{}.toml", player.get_name()), save_data) {
                tracing::error!("Failed to save player: {}", e);
            }
        }
        else {
            warn!("Player not found: {} while saving the game progression", player_id);
        }
    }

    pub fn get_player(&self, player_id: PlayerId) -> Option<&Player> {
        self.players.get(&player_id)
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
        if std::path::Path::new(path).exists() {
            if let Ok(server_save) = confy::load_path::<ServerSave>(path) {
                if server_save.next_player_id > 0 || server_save.next_item_id > 1 {
                    self.next_player_id = server_save.next_player_id;
                    self.next_item_id = server_save.next_item_id;
                    for (room_id_str, inventory) in server_save.rooms_inventory {
                        if let Ok(room_id) = room_id_str.parse::<u32>() {
                            if let Some(room) = self.all_rooms.get_mut(&room_id) {
                                room.set_inventory(inventory);
                            }
                        }
                    }
                } else {
                    self.next_player_id = 0;
                    self.next_item_id = (self.all_items.len() + 1) as ItemId;
                }
            } else {
                self.next_player_id = 0;
                self.next_item_id = (self.all_items.len() + 1) as ItemId;
            }
        } else {
            self.next_player_id = 0;
            self.next_item_id = (self.all_items.len() + 1) as ItemId;
        }
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
        if std::path::Path::new(&path).exists() {
            if let Ok(save_data) = confy::load_path::<crate::save::Save>(&path) {
                if save_data.name == name {
                    return Some(Player::from_save(save_data));
                }
            }
        }
        None
    }

    fn add_player_to_game(&mut self, player: Player) {
        let player_name = player.get_name().to_string();
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
            None => {
                error!("disconnect player: player not found");
                return;
            }
        };
        self.save_player(player_id);
        self.players.remove(&player_id);
        self.players_by_name.remove(&name);
    }

    pub fn get_nb_players(&self) -> usize {
        return self.players.len();
    }

    pub fn get_item_name(&self, item_id: &ItemId) -> String {
        self.all_items.get(item_id).unwrap().get_name().to_string()
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
            let key = player.as_str().unwrap().to_string();
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
                players.push(player.get_name().to_string());
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
        player.move_to_room(&room_name.to_string());
    }

    pub fn get_npcs_in_room_as_protocol_representations(&self, room_name: &str) -> Vec<String> {
        self.all_npcs
            .values()
            .filter(|&npc| npc.get_spawn_room() == room_name)
            .map(|npc| npc.get_protocol_representation())
            .collect()
    }

    pub fn get_npc(&self, npc_id: NpcId) -> Option<&Npc> {
        self.all_npcs.get(&npc_id)
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

    pub fn kill_npc(&mut self, npc_id: NpcId) {
        self.all_npcs.remove(&npc_id);
    }

    pub fn player_attacks_npc(&mut self, player_id: PlayerId, npc_id: NpcId) -> String {
        let player_damage = 10;
        let player_hp = self.get_player(player_id).unwrap().get_hp();
        let npc = self.get_mut_npc(npc_id).unwrap();
        let hp = npc.get_hp().unwrap();

        let new_npc_hp = if hp > player_damage {
            hp - player_damage
        } else {
            0
        };

        let status = if new_npc_hp > 0 { "combat" } else { "victory" };
        npc.set_hp(Some(new_npc_hp));
        if new_npc_hp == 0 {
            self.kill_npc(npc_id);
        }
        return format!(
            "{{\"attacker_hp\":{}, \"target_hp\":{}, \"damage\":{}, \"status\":\"{}\"}}",
            player_hp, new_npc_hp, player_damage, status
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

    pub fn get_room_id_from_name(&self, room_name: &str) -> RoomId {
        self.all_rooms
            .values()
            .find(|room| room.get_name() == room_name)
            .map(|room| room.get_id())
            .unwrap_or(2 as RoomId)
    }
}


