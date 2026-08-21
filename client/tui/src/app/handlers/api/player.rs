use crate::app::App;
use crate::states::game::Item;
use api_client::commands::{
    DropResponse, InventoryResponse, QuestResponse, QuestsResponse, StatusResponse, TakeResponse,
};

impl App {
    pub(crate) fn on_status(&mut self, response: StatusResponse) {
        self.state
            .game
            .player
            .set_vitals(response.player_status.hp, response.player_status.max_hp);

        self.state
            .game
            .log_action("You checked your state.".to_string());
    }

    pub(crate) fn on_inventory(&mut self, response: InventoryResponse) {
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

    pub(crate) fn on_quests(&mut self, response: QuestsResponse) {
        self.state.game.player.quests.set_items(response.quest_list);

        self.state
            .game
            .log_action("You checked your quests.".to_string());
    }

    pub(crate) fn on_quest(&mut self, response: QuestResponse) {
        self.state.game.player.quests.push(response.quest_data);
    }

    pub(crate) fn on_take_item(&mut self, response: TakeResponse) {
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

    pub(crate) fn on_drop_item(&mut self, response: DropResponse) {
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
}
