use std::collections::HashMap;
use crate::player::{Player, PlayerCount, PlayerId};
use crate::groups::{GroupManager};
use crate::items::{ItemId, Item};
use tracing::{error};
use json::{object};
pub struct GameManager {
    players: HashMap<PlayerId, Player>,
    players_by_name: HashMap<String, PlayerId>,
    groups: GroupManager,
    next_player_id: PlayerCount,
    next_item_id: ItemId,
    all_items: HashMap<ItemId, Item>
}

impl GameManager {
    pub fn new() -> Self {
        let mut manager = Self {
            players: HashMap::new(),
            players_by_name: HashMap::new(),
            groups: GroupManager::new(),
            next_player_id: 0,
            next_item_id: 0,
            all_items: HashMap::new()
        };
        manager.next_player_id = manager.restore_next_player_id();
        manager.next_item_id = manager.restore_next_item_id();
        return manager;
    }

    pub fn get_players(&self) -> &HashMap<PlayerId, Player> {
        return &self.players;
    }

    pub fn get_players_by_names(&self) -> &HashMap<String, PlayerId> {
        return &self.players_by_name;
    }

    pub fn all_groups(&mut self) -> &mut GroupManager {
        return &mut self.groups;
    }

    fn restore_next_player_id(&self) -> PlayerId {
        return 0;
    }

    fn restore_next_item_id(&self) -> ItemId {
        return 0;
    }
    fn change_player_name(&mut self, player_id: PlayerId, new_name: String) -> bool {
        
        if !self.players.contains_key(&player_id) {
            return false;
        }

        let player= match self.players.get_mut(&player_id) {
            Some(player) => player,
            _none => return false,
        };


        self.players_by_name.remove(player.get_name());
        player.set_name(new_name.clone());

        self.players_by_name.insert(new_name, player_id);

        return true;
    }

    fn try_restore_player_save(&mut self) -> Option<Player> 
    {
        // &mut self, name: String  
        Option::None
    }

    fn add_player_to_game(&mut self, player: Player) {
        let player_name = player.get_name().to_string();
        let player_id = player.get_id();
        self.players.insert(player_id, player);
        self.players_by_name.insert(player_name, player_id);
    }

    fn create_new_player(&mut self, name: String)
    {
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

    pub fn disconnect_player(&mut self, name: String){
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

    pub fn get_player_inventory(&self, player_name: &str) -> String{
        let player_id = self.players_by_name.get(player_name).unwrap();
        let player = self.players.get(player_id).unwrap();


        let items: Vec<String> = player.get_items().iter().map(|item_id| {format!(
                                                                                "item.{}.{}",
                                                                                item_id,
                                                                                self.get_item_name(item_id)
                                                                            )
                                                                }
                                                            ).collect();
        return object!{
            "items": items
        }.dump();
    }
}