use crate::app::App;
use crate::states::ui::Notification;
use crate::ui::views::editor::EditorView;
use crate::ui::views::game::GameView;
use api_client::events::{FightResultData, FightStartData, KillData};

impl App {
    pub(crate) fn on_kill(&mut self, kill_data: KillData) {
        let npc_name = self.state.game.manifest.npc_name(&kill_data.npc_id);
        let is_me = self.state.game.player.is_me(&kill_data.player);

        let message = if is_me {
            format!("You have defeated {}", npc_name)
        } else {
            format!("{} has been defeated by {}", npc_name, kill_data.player)
        };

        self.state.game.log_action(message);

        if let Some(room) = &mut self.state.game.room {
            room.remove_npc(&kill_data.npc_id);
        }
    }

    pub(crate) fn on_fight_start(&mut self, fight_data: FightStartData) {
        let npc_name = self.state.game.manifest.npc_name(&fight_data.npc_id);

        match EditorView::new(&fight_data) {
            Ok(view) => {
                self.state.game.fight.reset();
                self.state.game.overlays.close_all();
                self.state
                    .game
                    .log_action(format!("A fight started against {}.", npc_name));
                self.view_manager.set_view(Box::new(view));
            }
            Err(error) => {
                self.state.ui.notifications.push(Notification::error(error));
            }
        }
    }

    pub(crate) fn on_fight_result(&mut self, fight_result: FightResultData) {
        let is_me = self
            .state
            .game
            .player
            .is_me(fight_result.player_name.as_str());

        if is_me {
            self.state.game.fight.resolve(fight_result.success);

            if !fight_result.success {
                self.state
                    .game
                    .player
                    .take_damage(fight_result.damage_dealt);
            }
        }

        let who = match is_me {
            true => "You",
            false => fight_result.player_name.as_str(),
        };
        let verb = match fight_result.success {
            true => "dealt",
            false => "received",
        };
        let message = format!("{} {} {} damage", who, verb, fight_result.damage_dealt);

        let notification = match fight_result.success {
            true => Notification::success(message.as_str()),
            false => Notification::error(message.as_str()),
        };

        self.state.game.log_action(message);

        self.state
            .ui
            .notifications
            .push(notification.with_duration(3000));
    }

    pub(crate) fn on_fight_end(&mut self) {
        self.state.game.fight.reset();
        self.state.game.log_action("The fight ended.".to_string());
        self.view_manager.set_view(Box::new(GameView::new()));
    }
}
