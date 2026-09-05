use crate::app::runtime::App;
use crate::notification::Notification;
use client_api::ApiRequest;
use client_api::commands::{GroupCreateResponse, GroupJoinResponse, LookCommand};

impl App {
    pub fn on_group_created(&mut self, response: GroupCreateResponse) {
        let leader = self.state.game.player.name.clone().unwrap_or_default();

        self.state
            .game
            .group
            .join(response.group_id.clone(), leader);
        self.state
            .game
            .log_action(format!("You created group {}.", response.group_id));
    }

    pub fn on_group_joined(&mut self, response: GroupJoinResponse, leader: String) {
        self.state
            .game
            .group
            .join(response.group_id, leader.clone());
        self.state
            .game
            .log_action(format!("You joined the group of {}.", leader));
    }

    pub fn on_group_left(&mut self) {
        self.state.game.group.leave();
        self.state
            .game
            .log_action("You left the group.".to_string());
    }

    pub fn on_group_invite_sent(&mut self, username: String) {
        self.state.ui.notifications.push(Notification::info(format!(
            "Invitation sent to {}.",
            username
        )));

        self.state
            .game
            .log_action(format!("You invited {} to your group.", username));
    }

    pub fn on_group_invited_by(&mut self, leader: String) {
        self.state.ui.notifications.push(Notification::info(format!(
            "You are invited to a group by {}.",
            leader
        )));

        self.state.game.group.invited_by(leader);
    }

    pub fn on_group_invite_removed(&mut self, leader: String) {
        self.state.game.group.remove_invitation(&leader);
        self.state
            .game
            .log_action(format!("The invitation from {} was withdrawn.", leader));
    }

    pub fn on_group_member_joined(&mut self, user: String) {
        self.state
            .ui
            .notifications
            .push(Notification::info(format!("{} joined the group.", user)));

        self.state
            .game
            .log_action(format!("{} joined the group.", user));
    }

    pub fn on_group_member_left(&mut self, user: String) {
        if self.state.game.group.is_leader(Some(&user)) {
            self.state.game.group.leave();

            let message = format!("Leader {} left. The group has been disbanded.", user);

            self.state
                .ui
                .notifications
                .push(Notification::warning(message.clone()));

            self.state.game.log_action(message);
        } else {
            let message = format!("{} left the group.", user);

            self.state
                .ui
                .notifications
                .push(Notification::info(message.clone()));

            self.state.game.log_action(message);
        }
    }

    pub fn on_group_moved(&mut self, direction: String) {
        self.state.game.end_npc_interaction();
        self.state
            .game
            .log_action(format!("Group moved to {}.", direction));

        self.send(ApiRequest::Look(LookCommand));
    }
}
