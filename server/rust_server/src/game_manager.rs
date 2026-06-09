use std::collections::HashMap;
use crate::player::{Player, PlayerId, PlayerCount};
use crate::constantes::{ErrorCode};

pub struct GameManager {
    players: HashMap<PlayerId, Player>,
    players_by_name: HashMap<String, PlayerId>,
    next_player_id: PlayerCount
}

impl GameManager {
    pub fn new() -> Self {
        let mut manager = Self {
            players: HashMap::new(),
            players_by_name: HashMap::new(),
            next_player_id: 0,
        };
        manager.next_player_id = manager.restore_next_player_id();
        return manager;
    }

    fn restore_next_player_id(&mut self) -> PlayerId {
        return 0;
    }

    fn change_player_name(&mut self, player_id: PlayerId, new_name: String) -> bool {
        
        if !self.players.contains_key(&player_id) {
            return false;
        }

        let player = match self.players.get_mut(&player_id) {
            Some(player) => player,
            None => return false,
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

    fn create_new_player(&mut self, name: String)
    {
        let player_id = self.next_player_id;
        let player = Player::new(name, player_id);

        self.players.insert(player_id, player);
        self.next_player_id += 1;
    }

    pub fn init_player(&mut self, name: String) -> ErrorCode {
        // get the data of this player from the database and add the player to the game
        // init an empty save if the player never played before
        if self.players_by_name.contains_key(&name) {
            return ErrorCode::NameInUse
        }
        match self.try_restore_player_save() {
            Some(_player) => return ErrorCode::NoError,
            None => self.create_new_player(name),
        }
        return ErrorCode::NoError;
    }
}