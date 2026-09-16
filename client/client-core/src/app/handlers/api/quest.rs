use crate::app::App;
use crate::notification::{Notification, NotificationDuration};
use crate::states::game::Item;
use client_api::commands::{QuestData, QuestResponse, QuestsResponse};
use client_api::events::{QuestCompleteData, QuestStepData};
use std::time::Duration;

impl App {
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

    pub fn on_quest(&mut self, response: QuestResponse) {
        self.state.game.player.set_quest(response.quest_data);
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
}
