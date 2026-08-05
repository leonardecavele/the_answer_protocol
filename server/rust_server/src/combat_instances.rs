use crate::npc::NpcId;
use crate::player::PlayerId;
use std::collections::HashMap;

pub struct CombatInstanceManager {
    pub instances: HashMap<NpcId, CombatInstance>,
}

impl CombatInstanceManager {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    pub fn get_instance_for_player(&self, player_id: PlayerId) -> Option<&CombatInstance> {
        self.instances
            .values()
            .find(|instance| instance.grouped_players.contains(&player_id) || instance.leader == player_id)
    }

    pub fn get_mut_instance_for_player(&mut self, player_id: PlayerId) -> Option<&mut CombatInstance> {
        self.instances
            .values_mut()
            .find(|instance| instance.grouped_players.contains(&player_id) || instance.leader == player_id)
    }

    pub fn add_instance(&mut self, leader: PlayerId, npc_id: NpcId, players_ids: Vec<PlayerId>) {
        let instance = CombatInstance::new(npc_id, leader, players_ids);
        self.instances.insert(npc_id, instance);

    }
    pub fn remove_finished_instances(&mut self) {
        self.instances
            .retain(|_, instance| !instance.all_players_finished());
    }

}

pub struct CombatInstance {
    leader: PlayerId,
    grouped_players: Vec<PlayerId>,
    npc_id: NpcId,
    players_success: HashMap<PlayerId, Option<bool>>,
    // Option<bool> because the success is None until the player played
}

impl CombatInstance {
    pub fn new(npc_id: NpcId, leader: PlayerId, grouped_players: Vec<PlayerId>) -> Self {
        let player_success = grouped_players.iter()
                                            .map(|player| (*player, None))
                                            .collect::<HashMap<PlayerId, Option<bool>>>();
        Self {
            leader,
            grouped_players,
            npc_id,
            players_success: player_success,
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
    pub fn all_players_finished(&self) -> bool {
        self.players_success.values().all(|s| s.is_some())
    }

    pub fn set_player_success(&mut self, player_id: PlayerId, success: bool) {
        self.players_success.insert(player_id, Some(success));
    }

    pub fn get_player_success(&self, player_id: PlayerId) -> Option<Option<bool>> {
        self.players_success.get(&player_id).copied()
    }
    
}
