    use std::collections::HashMap;

    type PlayerId = u32;
    type PlayerCount 
    struct Player {
        name: String, 
        id: PlayerId,
    }

    struct GameManager {
        players: HashMap<PlayerId, Player>,
        players_by_name: HashMap<String, PlayerId>,
        next_player_id: u32
    }

    impl GameManager {
        fn change_player_name(&mut self, player_id: PlayerId, new_name: String) -> bool {
            
            if !self.players.contains_key(&player_id) {
                return false;
            }

            let player = match self.players.get_mut(&player_id) {
                Some(player) => player,
                None => return false,
            };


            self.players_by_name.remove(&player.name);
            player.name = new_name.clone();

            self.players_by_name.insert(new_name, player_id);

            return true;
        }

        fn init_player(&mut self, name: String){
            // get the data of this player from the database and add the player to the game
            // init an empty save if the player never played before

        }
    }


