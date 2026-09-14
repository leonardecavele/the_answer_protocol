use tracing::debug;

use crate::constants::{ITEM_DESPAWN_TIME, LOST_ITEM, LOST_ITEM_SPAWN, TICK_TIME, TickResult};
use crate::game_manager::GameManager;
use crate::items::ItemId;
use std::sync::mpsc;
use std::time::Instant;

impl GameManager {
    pub fn process_incoming_events(&mut self, tick_timer: Instant) -> TickResult {
        loop {
            // Process any pending responses from the code tester thread
            self.process_tester_responses();
            self.process_admin_commands();

            if tick_timer.elapsed() >= TICK_TIME {
                break;
            }
            match self.receive_data_timeout(TICK_TIME - tick_timer.elapsed()) {
                Ok(msg) => {
                    let command_response = self.handle_message(msg);
                    debug!("sent response to client: {}", command_response);
                    if self.send_msg_to_client(command_response) == TickResult::Exit {
                        return TickResult::Exit;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return TickResult::Exit,
            };
        }
        TickResult::TickEnd
    }

    pub fn update_game_state(&mut self) {
        self.remove_finished_combat_instances();

        self.check_finished_quests();
        self.punish_inactive_players_in_combat();
        self.revive_dead_npcs();
        self.spawn_items();

        let current_time = Instant::now();

        let mut actions: Vec<(String, ItemId, bool, String)> = Vec::new();

        for room in self.all_rooms.values() {
            for item_id in room.get_inventory().get_items() {
                if let Some(item) = self.get_item(*item_id)
                    && let Some(dropped_time) = item.get_dropped_at()
                    && current_time.duration_since(dropped_time) >= ITEM_DESPAWN_TIME
                {
                    let no_despawn_room = item.get_remove_despawn_in_room();
                    if no_despawn_room != Some(room.get_id()) {
                        actions.push((
                            room.get_name().to_owned(),
                            *item_id,
                            item.get_id() == (LOST_ITEM as ItemId),
                            item.get_protocol_representation(),
                        ));
                    }
                }
            }
        }
        for (room_name, item_id, is_lost_item, item_rep) in actions {
            self.remove_item_from_room(&room_name, item_id);
            self.reset_dropped_at_for_item(item_id);
            let players = self.get_all_players_at_room(&room_name);
            let data = format!("type={} id={}", "ITEM", item_rep);

            self.send_no_player_event(&players, "DESPAWN", &data);

            if is_lost_item {
                let lost_item_spawn_players = self.get_all_players_at_room(LOST_ITEM_SPAWN);
                self.add_item_to_room(LOST_ITEM_SPAWN, item_id);
                self.send_no_player_event(&lost_item_spawn_players, "SPAWN", &data);
            } else {
                // despawned from the world, we can recycle the ID
                self.recycle_item_id(item_id);
            }
        }
    }
}
