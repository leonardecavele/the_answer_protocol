use crate::app::App;
use crate::notification::{Notification, NotificationTopic};
use crate::states::game::Item;
use client_api::commands::{DropResponse, TakeResponse, UseResponse};

impl App {
    pub fn on_use(&mut self, response: UseResponse) {
        self.state.game.player.take_item(&response.id);

        match response.r#type.as_str() {
            "heal" => {
                let healed = response
                    .context
                    .get("healed")
                    .and_then(|amount| amount.parse::<u16>().ok());
                let health = response
                    .context
                    .get("health")
                    .and_then(|amount| amount.parse::<u16>().ok());

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
}
