use crate::game_manager::{GameManager};
use crate::player::PlayerId;
use crate::constantes::ErrorCode;
use std::collections::HashMap;
pub type GroupId = u32;


pub struct Group {
    leader: PlayerId,
    id: GroupId,
    players: Vec<PlayerId>
}

impl Group {
    pub fn new(leader: PlayerId, group_id: GroupId) -> Self {
        return  Self { leader, id: group_id, players: vec![leader]};
    }
    pub fn get_id(&self) -> GroupId {
        self.id
    }
    pub fn get_leader(&self) -> PlayerId {
        self.leader
    }
    pub fn add_player(&mut self, player_id: PlayerId) {
        /*
        add a player to the group
        */
        self.players.push(player_id);
    }
}

pub struct GroupManager {
    groups: HashMap<GroupId, Group>,
    next_group_id: GroupId,
}

impl GroupManager {
    pub fn new() -> Self {
        return Self {
            groups: HashMap::new(),
            next_group_id: 0,
        };
    }
    pub fn get_group(&self, group_id: GroupId) -> Option<&Group> {
        self.groups.get(&group_id)
    }
    fn add_group(&mut self, leader: PlayerId) {
        
        let group_id = self.next_group_id;

        let group = Group::new(leader, self.next_group_id);
        self.groups.insert(group_id, group);

        self.next_group_id += 1;
    }
}

impl GameManager {
    pub fn create_group(&mut self, group_leader: PlayerId) -> ErrorCode {
        let player = self.get_player(group_leader).unwrap();
        
        if player.get_group_id().is_some() {
            return ErrorCode::AlreadyInGroup;
        }
        self.all_groups().add_group(group_leader);
        return ErrorCode::NoError;
    }
}
