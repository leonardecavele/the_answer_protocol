use crate::constantes::{ITEM_DESPAWN_TIME, LOST_ITEM, LOST_ITEM_SPAWN, TICK_TIME, TickResult};
use crate::game_manager::GameManager;
use crate::items::ItemId;
use std::sync::mpsc;
use std::time::Instant;

impl GameManager {
    pub fn apply_players_changes(&mut self, tick_timer: Instant) -> std::io::Result<TickResult> {
        loop {
            // Process any pending responses from the code tester thread
            self.process_tester_responses()?;

            if tick_timer.elapsed() >= TICK_TIME {
                break;
            }
            match self.receive_data_timeout(TICK_TIME - tick_timer.elapsed()) {
                Ok(msg) => {
                    let command_response = self.handle_message(msg);
                    self.send_msg_to_client(command_response)?;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(TickResult::Exit),
            };
        }
        return Ok(TickResult::TickEnd);
    }

    pub fn update_game_state(&mut self) -> std::io::Result<()> {
        self.remove_finished_combat_instances();
        self.punish_inactive_players_in_combat();
        self.revive_dead_npcs();
        for quest_instance in self.quest_instances.iter_mut() {
            let _state = quest_instance.get_state();

            match quest_instance.get_quest_name().as_str() {
                "Tunnel" => {}
                _ => {}
            }
        }

        let current_time = Instant::now();

        let mut actions: Vec<(String, ItemId, bool, String)> = Vec::new();

        for room in self.all_rooms.values() {
            for item_id in room.get_inventory().get_items() {
                if let Some(item) = self.all_items.get(item_id) {
                    if let Some(dropped_time) = item.get_dropped_at() {
                        if current_time.duration_since(dropped_time) >= ITEM_DESPAWN_TIME {
                            let no_despawn_room = item.get_remove_despawn_in_room();
                            if no_despawn_room.is_none()
                                || no_despawn_room.unwrap() != room.get_id()
                            {
                                actions.push((
                                    room.get_name().to_string(),
                                    *item_id,
                                    item.get_id() == (LOST_ITEM as ItemId),
                                    item.get_protocol_representation(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        for (room_name, item_id, is_lost_item, item_rep) in actions {
            self.remove_item_from_room(&room_name, item_id);
            self.reset_dropped_at_for_item(item_id);
            let players = self.get_all_players_at_room(&room_name);
            let data = format!("type={} id={}", "ITEM", item_rep);

            let event_despawn =
                GameManager::generate_no_player_event_json(&players, "DESPAWN", &data);

            self.add_diff_to_tick(event_despawn);

            if is_lost_item {
                let lost_item_spawn_players = self.get_all_players_at_room(LOST_ITEM_SPAWN);
                self.add_item_to_room(LOST_ITEM_SPAWN, item_id);
                let event_spawn = GameManager::generate_no_player_event_json(
                    &lost_item_spawn_players,
                    "SPAWN",
                    &data,
                );
                self.add_diff_to_tick(event_spawn);
            }
        }

        Ok(())
    }
}
