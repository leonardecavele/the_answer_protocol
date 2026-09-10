use crate::app::App;
use crate::notification::{Notification, NotificationDuration, NotificationTopic};
use crate::states::game::Item;
use client_api::ApiRequest;
use client_api::commands::{
    DropResponse, InventoryResponse, LookCommand, QuestData, QuestResponse, QuestsResponse,
    StatusResponse, TakeResponse, UseResponse,
};
use client_api::events::{QuestCompleteData, QuestStepData};
use std::time::Duration;

impl App {
    pub fn on_use(&mut self, response: UseResponse) {
        self.state.game.player.take_item(&response.id);

        match response.r#type.as_str() {
            "heal" => {
                let healed = response
                    .context
                    .get("healed")
                    .and_then(|amount| amount.parse::<u32>().ok());
                let health = response
                    .context
                    .get("health")
                    .and_then(|amount| amount.parse::<u32>().ok());

                let (Some(healed), Some(health)) = (healed, health) else {
                    self.record_trace(
                        "desync",
                        format!(
                            "used an item with an unreadable heal context: {:?}",
                            response
                        ),
                    );

                    self.state.ui.notifications.push(
                        Notification::warning("The server sent an unreadable heal effect.")
                            .with_topic(NotificationTopic::Protocol),
                    );

                    return;
                };

                self.state.game.player.set_hp(health);

                let message = format!("You healed {} HP.", healed);

                self.state.game.log_action(message.clone());
                self.state
                    .ui
                    .notifications
                    .push(Notification::success(message));
            }
            effect => {
                self.record_trace("desync", format!("used an unknown effect: {}", effect));

                self.state.ui.notifications.push(
                    Notification::warning(format!("The {} effect is not handled yet.", effect))
                        .with_topic(NotificationTopic::Protocol),
                );
            }
        }
    }

    pub fn on_status(&mut self, response: StatusResponse) {
        self.state
            .game
            .player
            .set_vitals(response.player_status.hp, response.player_status.max_hp);

        self.state.game.log_action(format!(
            "You checked your status. You have {} HP remaining.",
            response.player_status.hp
        ));
    }

    pub fn on_inventory(&mut self, response: InventoryResponse) {
        let items_count = response.inventory.len();
        self.state.game.player.set_inventory(
            response
                .inventory
                .into_iter()
                .map(|id| Item::from_manifest(id, &self.state.game.manifest))
                .collect(),
        );

        self.state.game.log_action(format!(
            "You checked your inventory. You have {} items in your inventory.",
            items_count
        ));
    }

    pub fn on_quests(&mut self, response: QuestsResponse) {
        let active_quests_count = response
            .quest_list
            .iter()
            .filter(|q| !q.is_completed())
            .count();
        let completed_quests_count = response.quest_list.len() - active_quests_count;

        self.state.game.player.set_quests(response.quest_list);

        self.state.game.log_action(format!(
            "You checked your quests. You have {} active quests and {} completed quests.",
            active_quests_count, completed_quests_count
        ));
    }

    pub fn on_quest_add(&mut self, data: QuestData) {
        self.state
            .ui
            .notifications
            .push(Notification::info("New quest added".to_string()));

        self.state.game.player.set_quest(data);
    }

    pub fn on_quest_step(&mut self, response: QuestStepData) {
        self.state.game.log_action(format!(
            "Quest {} next step reached ! ({} -> {})",
            response.name,
            response.current_step.saturating_sub(1),
            response.current_step
        ));

        self.state.ui.notifications.push(
            Notification::success(format!(
                "Quest {}\nnext step reached ! ({} -> {})",
                response.name,
                response.current_step.saturating_sub(1),
                response.current_step
            ))
            .with_duration(NotificationDuration::Finite(Duration::from_millis(12_000))),
        );

        self.state
            .game
            .player
            .set_quest_step(response.name, response.current_step);
    }

    pub fn on_quest_complete(&mut self, response: QuestCompleteData) {
        self.state
            .game
            .log_action(format!("Quest {} completed!", response.name));

        self.state.ui.notifications.push(
            Notification::success(format!("Quest {}\ncompleted!", response.name))
                .with_duration(NotificationDuration::Finite(Duration::from_millis(12_000))),
        );

        let items = response
            .reward_items
            .iter()
            .map(|name| Item::from_manifest(name.clone(), &self.state.game.manifest))
            .collect();

        self.state
            .game
            .player
            .set_quest_as_completed(response.name, items);
    }

    pub fn on_quest(&mut self, response: QuestResponse) {
        self.state.game.player.set_quest(response.quest_data);
    }

    pub fn on_item_add(&mut self, item_id: String) {
        let item = Item::from_manifest(item_id, &self.state.game.manifest);
        let message = "The item fell from the sky and landed in your inventory".to_string();

        self.state
            .ui
            .notifications
            .push(Notification::info(message.clone()));

        self.state.game.log_action(message);

        self.state.game.player.add_item(item);
    }

    pub fn on_take_item(&mut self, response: TakeResponse) {
        let id = response.item_identifier;

        let taken = self
            .state
            .game
            .room
            .as_mut()
            .and_then(|room| room.take_item(&id));

        let item = match taken {
            Some(item) => item,
            None => {
                self.record_trace("desync", format!("took {} which was not in the room", id));
                Item::from_manifest(id, &self.state.game.manifest)
            }
        };

        self.state
            .game
            .log_action(format!("You took {}.", item.name));
        self.state.game.player.add_item(item);
    }

    pub fn on_drop_item(&mut self, response: DropResponse) {
        let id = response.item_identifier;

        let item = match self.state.game.player.take_item(&id) {
            Some(item) => item,
            None => {
                self.record_trace(
                    "desync",
                    format!("dropped {} which was not in the player inventory", id),
                );
                Item::from_manifest(id, &self.state.game.manifest)
            }
        };

        self.state
            .game
            .log_action(format!("You dropped {}.", item.name));

        if let Some(room) = &mut self.state.game.room {
            room.spawn_item(item);
        }
    }

    pub fn on_teleport(&mut self) {
        self.state.ui.notifications.push(Notification::warning(
            "A player on your team died, you respawn with them.",
        ));

        self.send(ApiRequest::Look(LookCommand))
    }
}
