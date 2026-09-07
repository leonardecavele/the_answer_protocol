use crate::app::App;
use crate::notification::{Notification, NotificationDuration};
use crate::states::game::Item;
use client_api::ApiRequest;
use client_api::commands::{
    DropResponse, InventoryResponse, LookCommand, QuestResponse, QuestsResponse, StatusResponse,
    TakeResponse,
};
use client_api::events::{QuestCompleteData, QuestStepData};
use std::time::Duration;

impl App {
    pub fn on_status(&mut self, response: StatusResponse) {
        self.state
            .game
            .player
            .set_vitals(response.player_status.hp, response.player_status.max_hp);

        self.state
            .game
            .log_action("You checked your state.".to_string());
    }

    pub fn on_inventory(&mut self, response: InventoryResponse) {
        self.state.game.player.inventory.set_items(
            response
                .inventory
                .into_iter()
                .map(|id| Item::from_manifest(id, &self.state.game.manifest))
                .collect(),
        );

        self.state
            .game
            .log_action("You checked your inventory.".to_string());
    }

    pub fn on_quests(&mut self, response: QuestsResponse) {
        self.state.game.player.quests.set_items(response.quest_list);

        self.state
            .game
            .log_action("You checked your quests.".to_string());
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
        self.state.game.player.quests.push(response.quest_data);
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
        self.state.game.player.inventory.push(item);
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
            "💀 A player on your team absolutely sucks. 🤬",
        ));

        self.send(ApiRequest::Look(LookCommand))
    }
}
