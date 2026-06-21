use crate::constantes::Direction;
use crate::items::{Item, ItemId};
use crate::npc::{Npc, NpcId};
use crate::parser::Parser;
use crate::inventory::Inventory;
use crate::player::{Player, PlayerCount, PlayerId};
use crate::room::{Room, RoomName};
use json::JsonValue;
use json::object;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use tracing::error;

pub struct GameManager {
    players: HashMap<PlayerId, Player>,
    players_by_name: HashMap<String, PlayerId>,
    next_player_id: PlayerCount,
    next_item_id: ItemId,
    all_items: HashMap<ItemId, Item>,
    all_rooms: HashMap<RoomName, Room>,
    all_npcs: HashMap<NpcId, Npc>,
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
        let mut starting_items = HashMap::new();
        let item_id: ItemId = 0;
        let item = Item::new(
            item_id,
            "test sword".to_string(),
            "A sword made for testing".to_string(),
        );

        starting_items.insert(item_id, item);
        let mut starting_rooms: HashMap<String, Room> = HashMap::new();
        let room_name: RoomName = "room_test".to_string();
        let room_name2: RoomName = "room_test2".to_string();

        let mut room_exits = HashMap::new();
        room_exits.insert("NORTH".to_string(), room_name2.clone());

        let mut room_exits2 = HashMap::new();
        room_exits2.insert("SOUTH".to_string(), room_name.clone());

        let mut room = Room::new(0, room_name.clone(), "nothing".to_string(), room_exits);
        let room2 = Room::new(1, room_name2.clone(), "nothing".to_string(), room_exits2);
        room.add_item(item_id);
        starting_rooms.insert(room_name, room);
        starting_rooms.insert(room_name2, room2);

        let mut manager = Self {
            players: HashMap::new(),
            players_by_name: HashMap::new(),
            next_player_id: 0,
            next_item_id: 0,
            all_items: starting_items,
            all_rooms: starting_rooms,
            all_npcs: parser.get_npcs().clone(),
            mpsc_receiver,
            writer_stream,
            tick_diff: HashMap::new(),
        };

        manager.next_player_id = manager.restore_next_player_id();
        manager.next_item_id = manager.restore_next_item_id();
        return manager;
    }

    pub fn get_players(&self) -> &HashMap<PlayerId, Player> {
        return &self.players;
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

    fn restore_next_player_id(&self) -> PlayerId {
        return 0;
    }

    fn restore_next_item_id(&self) -> ItemId {
        return 0;
    }
    pub fn get_all_items(&mut self) -> &mut HashMap<ItemId, Item> {
        return &mut self.all_items;
    }

    // fn change_player_name(&mut self, player_id: PlayerId, new_name: String) -> bool {
    //     if !self.players.contains_key(&player_id) {
    //         return false;
    //     }

    //     let player = match self.players.get_mut(&player_id) {
    //         Some(player) => player,
    //         _none => return false,
    //     };

    //     self.players_by_name.remove(player.get_name());
    //     player.set_name(new_name.clone());

    //     self.players_by_name.insert(new_name, player_id);

    //     return true;
    // }

    fn try_restore_player_save(&mut self) -> Option<Player> {
        // &mut self, name: String
        Option::None
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
        // get the data of this player from the database and add the player to the game
        // init an empty save if the player never played before
        match self.try_restore_player_save() {
            Some(_player) => return,
            _none => self.create_new_player(name),
        }
    }

    pub fn disconnect_player(&mut self, name: String) {
        let player_id_wrapped = self.players_by_name.get(&name);
        if player_id_wrapped.is_none() {
            error!("disconnect player: player not found");
        }
        let player_id = player_id_wrapped.unwrap();
        self.players.remove(player_id);
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
        return object! {
            "items": items
        }
        .dump();
    }

    pub fn get_room(&self, room_name: &str) -> Option<&Room> {
        self.all_rooms.get(room_name)
    }

    pub fn remove_item_from_room(&mut self, room_name: &str, item_id: ItemId) {
        let room = self.all_rooms.get_mut(room_name).unwrap();
        room.remove_item(item_id);
    }

    pub fn add_item_to_room(&mut self, room_name: &str, item_id: ItemId) {
        let room = self.all_rooms.get_mut(room_name).unwrap();
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

    pub fn get_neighbor_room_name(
        &self,
        room_name: &str,
        direction: &Direction,
    ) -> Option<&RoomName> {
        let room = self.get_room(room_name);
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

    pub fn move_player_to_room(&mut self, player_name: &str, room_name: &str) {
        let player = self.get_mut_player_from_name(player_name).unwrap();
        player.move_to_room(&room_name.to_string());
    }

    pub fn get_npcs_in_room(&self, room_name: &str) -> Vec<String> {
        self.all_npcs
            .values()
            .filter(|&npc| npc.get_spawn_room() == room_name)
            .map(|npc| npc.get_name().to_string())
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
}
