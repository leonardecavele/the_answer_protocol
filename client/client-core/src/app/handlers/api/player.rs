use crate::app::App;
use crate::notification::Notification;
use crate::states::game::Item;
use client_api::ApiRequest;
use client_api::commands::{InventoryResponse, LookCommand, StatusResponse};

impl App {
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

    pub fn on_teleport(&mut self) {
        self.state.ui.notifications.push(Notification::warning(
            "A player on your team died, you respawn with them.",
        ));

        self.send(ApiRequest::Look(LookCommand))
    }
}
