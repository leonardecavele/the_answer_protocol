use crate::app::App;
use crate::network::RequestChain;
use crate::notification::{Notification, NotificationTopic};
use crate::renderer::views::{EditorView, GameView};
use crate::states::game::DialogueState;
use client_api::ApiRequest;
use client_api::commands::{AttackResponse, LookCommand, StatusCommand};
use client_api::events::{
    CounterAttackData, DeathData, FightEndData, FightResultData, FightStartData, KillData,
};

impl App {
    pub fn on_death(&mut self, death: DeathData) {
        let is_me = self.state.game.player.is_me(&death.player_name);
        let respawn_here = self
            .state
            .game
            .room
            .as_ref()
            .is_some_and(|room| room.id == death.respawn_room_id);

        if !is_me && let Some(room) = &mut self.state.game.room {
            if respawn_here {
                room.player_entered(death.player_name.clone());
            } else {
                room.player_left(&death.player_name);
            }
        }

        let message = match (is_me, respawn_here) {
            (true, true) => "You died and respawned here".to_string(),
            (true, false) => {
                format!("You died and respawned in {}", death.respawn_room_id)
            }
            (false, true) => {
                format!("{} died and respawned here", death.player_name)
            }
            (false, false) => format!(
                "{} died and respawned in {}",
                death.player_name, death.respawn_room_id
            ),
        };

        if is_me {
            self.send_chain(RequestChain::new(vec![
                ApiRequest::Look(LookCommand),
                ApiRequest::Status(StatusCommand),
            ]));
        }

        self.state.game.log_action(message);
    }

    pub fn on_counter_attack(&mut self, counter_attack: CounterAttackData) {
        let npc_name = self.state.game.manifest.npc_name(&counter_attack.npc_id);
        let message = format!(
            "{} dealt {} damage to you.",
            npc_name, counter_attack.dealt_damage
        );

        self.state.game.log_action(message.clone());
        self.state
            .ui
            .notifications
            .push(Notification::error(message));

        self.state.game.player.set_hp(counter_attack.current_hp);
    }

    pub fn on_kill(&mut self, kill: KillData) {
        let npc_name = self.state.game.manifest.npc_name(&kill.npc_id);
        let is_me = self.state.game.player.is_me(&kill.player);

        let message = if is_me {
            format!("You have defeated {}", npc_name)
        } else {
            format!("{} has been defeated by {}", npc_name, kill.player)
        };

        self.state.game.log_action(message);

        if let Some(room) = &mut self.state.game.room {
            room.remove_npc(&kill.npc_id);
        }
    }

    pub fn on_fight_start(&mut self, fight_start: FightStartData) {
        let npc_name = self.state.game.manifest.npc_name(&fight_start.npc_id);

        match EditorView::new(&fight_start) {
            Ok(view) => {
                self.state
                    .game
                    .fight
                    .start(fight_start.npc_hp, fight_start.npc_max_hp);
                self.state.game.close_all_overlays();
                self.state
                    .game
                    .log_action(format!("A fight started against {}.", npc_name));
                self.view_manager.set_view(Box::new(view));
            }
            Err(error) => {
                self.state
                    .ui
                    .notifications
                    .push(Notification::error(error).with_topic(NotificationTopic::Fight));
            }
        }
    }

    pub fn on_fight_result(&mut self, fight_result: FightResultData) {
        let is_me = self
            .state
            .game
            .player
            .is_me(fight_result.player_name.as_str());

        if is_me {
            self.state.game.fight.resolve(fight_result.success);

            if !fight_result.success {
                self.state.game.player.set_hp(fight_result.current_hp);
            }
        }

        if fight_result.success {
            self.state.game.fight.set_npc_hp(fight_result.damage_dealt);
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

        self.state.ui.notifications.push(notification.with_ms(3000));
    }

    pub fn on_fight_end(&mut self, _fight_end: FightEndData) {
        //TODO: ajouter les donnees du combat dans l'etat globale
        //TODO: afficher sur la vue du jeu un access a ces donnees (clique souris + clavier)
        //TODO: permettre de selectionner un combat dans la liste et mettre en forme le resultat (popup ?)

        self.state.game.fight.end();
        self.state.game.log_action("The fight ended.".to_string());
        self.view_manager.set_view(Box::new(GameView::new()));
    }

    pub fn on_fight_timed_out(&mut self) {
        self.state.ui.notifications.push(
            Notification::warning("The server never ended the fight. Leaving the editor.")
                .with_topic(NotificationTopic::Fight),
        );

        self.state.game.fight.end();
        self.state
            .game
            .log_action("The fight ended. (timeout)".to_string());
        self.view_manager.set_view(Box::new(GameView::new()));
    }

    pub fn on_attacked(&mut self, response: AttackResponse, npc_id: String) {
        let npc_name = self.state.game.manifest.npc_name(&npc_id);
        let combat = response.combat_result;

        self.state.game.inspected_npc = Some(npc_id.clone());
        self.state.game.player.set_hp(combat.attacker_hp);

        self.state
            .game
            .log_action(format!("You attacked {}.", npc_name));

        let text = format!(
            "Combat with {}: You dealt {} damage. (Your HP: {} | Target HP: {}) ",
            npc_name, combat.damage, combat.attacker_hp, combat.target_hp
        );

        self.state
            .game
            .open_dialogue(DialogueState::new(npc_id, npc_name, text.clone(), true));

        self.state.game.log_action(text);
    }
}
