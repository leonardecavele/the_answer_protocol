use crate::app::runtime::App;
use crate::states::notification::Notification;
use api_client::ApiRequest;
use api_client::commands::{GroupCreateResponse, GroupJoinResponse, LookCommand};

impl App {
    pub(crate) fn on_group_created(&mut self, response: GroupCreateResponse) {
        let leader = self.state.game.player.name.clone().unwrap_or_default();

        self.state
            .game
            .group
            .join(response.group_id.clone(), leader);
        self.state
            .game
            .log_action(format!("You created group {}.", response.group_id));
    }

    pub(crate) fn on_group_joined(&mut self, response: GroupJoinResponse, leader: String) {
        self.state
            .game
            .group
            .join(response.group_id, leader.clone());
        self.state
            .game
            .log_action(format!("You joined the group of {}.", leader));
    }

    pub(crate) fn on_group_left(&mut self) {
        self.state.game.group.leave();
        self.state
            .game
            .log_action("You left the group.".to_string());
    }

    pub(crate) fn on_group_invited_by(&mut self, leader: String) {
        self.state.ui.notifications.push(Notification::info(format!(
            "You are invited to a group by {}.",
            leader
        )));
    }

    pub(crate) fn on_group_member_joined(&mut self, user: String) {
        self.state
            .game
            .log_action(format!("{} joined the group.", user));
    }

    pub(crate) fn on_group_member_left(&mut self, user: String) {
        if self.state.game.group.is_leader(Some(&user)) {
            self.state.game.group.leave();
            self.state.game.log_action(format!(
                "Leader {} left. The group has been disbanded.",
                user
            ));
        } else {
            self.state
                .game
                .log_action(format!("{} left the group.", user));
        }
    }

    pub(crate) fn on_group_moved(&mut self, direction: String) {
        self.state
            .game
            .log_action(format!("Group moved to {}.", direction));

        self.send(ApiRequest::Look(LookCommand));
    }
}
