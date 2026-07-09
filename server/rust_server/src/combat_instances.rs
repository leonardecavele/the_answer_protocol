use crate::npc::NpcId;
use crate::player::PlayerId;
use std::collections::HashMap;

pub struct CombatInstanceManager {
    pub instances: HashMap<NpcId, Vec<PlayerId>>,
}

impl CombatInstanceManager {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    pub fn add_instance(&mut self, npc_id: NpcId, players_ids: Vec<PlayerId>) {
        self.instances.insert(npc_id, players_ids);
    }

    pub fn update_instance_player_left(player: PlayerId, npc_id: NpcId) {
        if let Some(players) = self.instances.get_mut(&npc_id) {
            players.retain(|id| *id != player);
            
        }
    }

}

pub struct CombatInstance {
    leader: PlayerId,
    grouped_players: Vec<PlayerId>,
    npc_id: NpcId,
}
impl CombatInstance {
    pub fn new(npc_id: NpcId, leader:  PlayerId, grouped_players: Vec<PlayerId>) -> Self {
        Self {
            leader,
            grouped_players,
            npc_id,
        }
    }
    
    pub fn get_leader(&self) -> PlayerId {
        self.leader
    }
    pub fn get_grouped_players(&self) -> &Vec<PlayerId> {
        &self.grouped_players
    }
    pub fn get_npc_id(&self) -> NpcId {
        self.npc_id
    }
}