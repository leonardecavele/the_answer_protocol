use crate::app::runtime::App;
use crate::states::notification::Notification;
use crate::ui::views::game::GameView;
use api_client::commands::{ConnectResponse, WhoResponse};

impl App {
    pub(crate) fn on_connected(&mut self, response: ConnectResponse) {
        self.state.game.player.set_name(response.player_name);
    }

    pub(crate) fn on_who(&mut self, response: WhoResponse) {
        self.state
            .game
            .server
            .set_online_count(response.player_count);
        self.state
            .game
            .log_action("You checked who is here.".to_string());
    }

    pub(crate) fn on_player_joined_server(&mut self, name: String) {
        // TODO: retirer cette estimation quand le serveur emettra STATS a chaque changement
        let count = self
            .state
            .game
            .server
            .online_players_count
            .saturating_add(1);
        self.state.game.server.set_online_count(count);

        self.state
            .game
            .log_action(format!("{} joined the server.", name));
    }

    pub(crate) fn on_player_quit_server(&mut self, name: String) {
        // TODO: retirer cette estimation quand le serveur emettra STATS a chaque changement
        let count = self
            .state
            .game
            .server
            .online_players_count
            .saturating_sub(1);
        self.state.game.server.set_online_count(count);

        if let Some(room) = &mut self.state.game.room {
            room.player_left(&name);
        }

        if self.state.game.group.is_leader(Some(&name)) {
            self.state.game.group.leave();
        }

        self.state
            .game
            .log_action(format!("{} disconnected.", name));
    }

    pub(crate) fn on_stats(&mut self, online_count: u32) {
        self.state.game.server.set_online_count(online_count);
    }

    pub(crate) fn on_unknown_event(&mut self, raw: String) {
        self.state
            .ui
            .notifications
            .push(Notification::warning(format!("Unknown event: {}", raw)));
    }

    pub(crate) fn on_game_server_connected(&mut self) {
        self.state
            .game
            .log_action("Game server online.".to_string());

        self.state.ui.notifications.push(Notification::info(
            "Game server is online. Session restarted.",
        ));

        self.state.network.is_connected = true;

        self.load_state_from_server();
    }

    pub(crate) fn on_game_server_disconnected(&mut self) {
        self.state
            .game
            .log_action("Game server offline.".to_string());

        if self.state.network.is_connected {
            self.view_manager.set_view(Box::new(GameView::new()));
        }

        self.state.game.close_all_overlays();
        self.state.network.is_connected = false;
    }
}
