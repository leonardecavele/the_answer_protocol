// use crate::npc::NpcId;
// use crate::player::PlayerId;
// use std::collections::HashMap;

// pub struct CombatInstanceManager {
//     instances: HashMap<NpcId, Vec<PlayerId>>,
// }


// impl CombatInstanceManager {
//     pub fn new(npc_id: NpcId, players_ids: Vec<PlayerId>) -> Option<Self> {
//         let mut instances = HashMap::new();
//         instances.insert(npc_id, players_ids);
//         Some(Self {
//             instances,
//         })
//     }
//     pub fn update_instance_player_left(player: PlayerId, npc_id: NpcId) {
//         if let Some(players) = self.instances.get_mut(&npc_id) {
//             players.retain(|id| *id != player);
            
//         }
//     }
// }