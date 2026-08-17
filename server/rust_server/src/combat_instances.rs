use crate::npc::NpcId;
use crate::player::PlayerId;
use std::collections::HashMap;
use std::time::Instant;

pub struct CombatInstanceManager {
    pub instances: HashMap<NpcId, CombatInstance>,
}

impl CombatInstanceManager {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    pub fn get_instance_for_npc(&self, npc_id: NpcId) -> Option<&CombatInstance> {
        self.instances.get(&npc_id)
    }

    pub fn get_instance_for_player(&self, player_id: PlayerId) -> Option<&CombatInstance> {
        self.instances.values().find(|instance| {
            instance.grouped_players.contains(&player_id) || instance.leader == player_id
        })
    }

    pub fn get_all_players_in_combat(&self, npc_id: NpcId) -> Vec<PlayerId> {
        let mut vec = Vec::new();
        let instance = self.instances.get(&npc_id).unwrap();
        vec.extend(instance.get_grouped_players());
        vec.push(instance.leader);
        return vec;
    }

    pub fn get_mut_instance_for_player(
        &mut self,
        player_id: PlayerId,
    ) -> Option<&mut CombatInstance> {
        self.instances.values_mut().find(|instance| {
            instance.grouped_players.contains(&player_id) || instance.leader == player_id
        })
    }

    pub fn add_instance(
        &mut self,
        leader: PlayerId,
        npc_id: NpcId,
        npc_hp: u32,
        players_ids: Vec<PlayerId>,
        file_name: String,
    ) {
        let instance = CombatInstance::new(npc_id, leader, npc_hp, players_ids, file_name);
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
    pub players_success: HashMap<PlayerId, Option<bool>>,
    npc_combat_start_hp: u32,
    pub combat_start_time: Instant, // Option<bool> because the success is None until the player played
    file_name: String,
}

impl CombatInstance {
    pub fn new(
        npc_id: NpcId,
        leader: PlayerId,
        npc_hp: u32,
        grouped_players: Vec<PlayerId>,
        file_name: String,
    ) -> Self {
        let mut player_success = grouped_players
            .iter()
            .map(|player| (*player, None))
            .collect::<HashMap<PlayerId, Option<bool>>>();
        player_success.insert(leader, None);
        Self {
            leader,
            grouped_players,
            npc_id,
            players_success: player_success,
            npc_combat_start_hp: npc_hp,
            combat_start_time: Instant::now(),
            file_name,
        }
    }

    pub fn get_leader(&self) -> PlayerId {
        self.leader
    }
    pub fn get_grouped_players(&self) -> &Vec<PlayerId> {
        &self.grouped_players
    }

    pub fn get_assigned_file_name(&self) -> &str {
        &self.file_name
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

    pub fn get_npc_combat_start_hp(&self) -> u32 {
        self.npc_combat_start_hp
    }
}
