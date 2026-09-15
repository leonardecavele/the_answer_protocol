use crate::npc::NpcId;
use crate::player::PlayerId;
use std::collections::HashMap;
use std::time::Instant;
use tracing::warn;

pub struct CombatInstanceManager {
    pub instances: HashMap<NpcId, CombatInstance>,
}

impl Default for CombatInstanceManager {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn player_is_in_instance(&self, player_id: PlayerId) -> bool {
        self.get_instance_for_player(player_id).is_some()
    }
    pub fn get_instance_for_player(&self, player_id: PlayerId) -> Option<&CombatInstance> {
        self.instances.values().find(|instance| {
            instance.grouped_players.contains(&player_id) || instance.leader == player_id
        })
    }

    pub fn get_all_players_in_combat(&self, npc_id: NpcId) -> Vec<PlayerId> {
        let mut vec = Vec::new();
        if let Some(instance) = self.instances.get(&npc_id) {
            vec.extend(instance.get_grouped_players());
            vec.push(instance.leader);
            vec
        } else {
            warn!("No combat instance found for npc_id: {}", npc_id);
            vec
        }
    }

    pub fn get_mut_instance_for_npc(&mut self, npc_id: NpcId) -> Option<&mut CombatInstance> {
        self.instances.get_mut(&npc_id)
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

#[derive(Debug, Clone, Default)]
pub struct PlayerCombatInfo {
    pub success: bool,
    pub response_time: u64,
    pub code: String,
    pub damage_dealt: u32,
}

pub struct CombatInstance {
    leader: PlayerId,
    grouped_players: Vec<PlayerId>,
    npc_id: NpcId,
    pub players_info: HashMap<PlayerId, Option<PlayerCombatInfo>>,
    npc_combat_start_hp: u32,
    pub combat_start_time: Instant,
    file_name: String,
    pub evaluating_players_count: u32,
    left_players: Vec<PlayerId>,
    died_players: Vec<PlayerId>,
}

impl CombatInstance {
    pub fn new(
        npc_id: NpcId,
        leader: PlayerId,
        npc_hp: u32,
        grouped_players: Vec<PlayerId>,
        file_name: String,
    ) -> Self {
        let mut players_info = grouped_players
            .iter()
            .map(|player| (*player, None))
            .collect::<HashMap<PlayerId, Option<PlayerCombatInfo>>>();
        players_info.insert(leader, None);
        Self {
            leader,
            grouped_players,
            npc_id,
            players_info,
            npc_combat_start_hp: npc_hp,
            combat_start_time: Instant::now(),
            file_name,
            evaluating_players_count: 0,
            left_players: Vec::new(),
            died_players: Vec::new(),
        }
    }

    pub fn player_died(&mut self, player_id: PlayerId) {
        self.died_players.push(player_id);
    }

    pub fn get_leader(&self) -> PlayerId {
        self.leader
    }
    pub fn get_grouped_players(&self) -> &Vec<PlayerId> {
        &self.grouped_players
    }

    pub fn get_combat_duration_in_seconds(&self) -> u64 {
        self.combat_start_time.elapsed().as_secs()
    }

    pub fn get_combat_duration_in_ms(&self) -> u64 {
        self.combat_start_time.elapsed().as_millis() as u64
    }

    pub fn get_assigned_file_name(&self) -> &str {
        &self.file_name
    }

    pub fn get_npc_id(&self) -> NpcId {
        self.npc_id
    }

    pub fn player_left_group(&mut self, player_id: PlayerId) {
        self.left_players.push(player_id);
    }

    pub fn force_finish(&mut self) {
        for info in self.players_info.values_mut() {
            if info.is_none() {
                *info = Some(PlayerCombatInfo {
                    success: true,
                    ..Default::default()
                });
            }
        }
    }
    pub fn get_died_players(&self) -> &Vec<PlayerId> {
        &self.died_players
    }

    pub fn get_left_players(&self) -> &Vec<PlayerId> {
        &self.left_players
    }

    pub fn all_players_finished(&self) -> bool {
        self.players_info.values().all(|s| s.is_some())
    }

    pub fn set_player_info(&mut self, player_id: PlayerId, info: PlayerCombatInfo) {
        if let Some(player_info) = self.players_info.get_mut(&player_id) {
            *player_info = Some(info);
        }
    }

    pub fn set_player_success(&mut self, player_id: PlayerId, success: bool) {
        if let Some(info) = self.players_info.get_mut(&player_id) {
            if let Some(combat_info) = info {
                combat_info.success = success;
            } else {
                *info = Some(PlayerCombatInfo {
                    success,
                    ..Default::default()
                });
            }
        }
    }

    pub fn get_player_success(&self, player_id: PlayerId) -> Option<Option<bool>> {
        self.players_info
            .get(&player_id)
            .map(|info| info.as_ref().map(|i| i.success))
    }

    pub fn get_npc_combat_start_hp(&self) -> u32 {
        self.npc_combat_start_hp
    }
    pub fn get_all_players(&self) -> Vec<PlayerId> {
        let mut vec = self.grouped_players.clone();
        vec.push(self.leader);
        vec
    }
}
