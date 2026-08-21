use crate::app::App;
use crate::states::game::{Item, Npc, Room};
use api_client::ApiRequest;
use api_client::commands::{LookCommand, LookResponse, StatusCommand};
use api_client::events::{DeathData, SpawnData};

impl App {
    pub(crate) fn on_look(&mut self, look: LookResponse) {
        let manifest = &self.state.game.manifest;
        let self_name = self.state.game.player.name.as_deref();

        let mut players = look.players;
        players.retain(|player| Some(player.as_str()) != self_name);

        let room = Room {
            id: look.room.id,
            name: look.room.name,
            description: look.room.description,
            exits: look.room.exits.into(),
            players,
            npcs: look
                .npcs
                .into_iter()
                .map(|id| Npc::from_manifest(id, manifest))
                .collect(),
            items: look
                .items
                .into_iter()
                .map(|id| Item::from_manifest(id, manifest))
                .collect(),
        };

        self.state
            .game
            .log_action(format!("You looked at {}.", room.name));
        self.state.game.room = Some(room);
    }

    pub(crate) fn on_moved(&mut self, direction: String) {
        self.state.game.overlays.inspected_entity = None;
        self.state
            .game
            .log_action(format!("You moved {}.", direction));
        self.handle_request(ApiRequest::Look(LookCommand));
    }

    pub(crate) fn on_npc_spawned(&mut self, spawn_data: SpawnData) {
        let npc = Npc::from_manifest(spawn_data.id, &self.state.game.manifest);

        let Some(room) = &mut self.state.game.room else {
            return;
        };

        let name = npc.name.clone();
        room.spawn_npc(npc);

        self.state.game.log_action(format!("{} has respawn", name));
    }

    pub(crate) fn on_item_spawned(&mut self, spawn_data: SpawnData) {
        let item = Item::from_manifest(spawn_data.id, &self.state.game.manifest);

        let Some(room) = &mut self.state.game.room else {
            return;
        };

        let name = item.name.clone();
        room.spawn_item(item);

        self.state
            .game
            .log_action(format!("{} has been catapulted here", name));
    }

    pub(crate) fn on_item_despawned(&mut self, spawn_data: SpawnData) {
        let Some(room) = &mut self.state.game.room else {
            return;
        };

        let Some(item) = room.take_item(&spawn_data.id) else {
            return;
        };

        self.state
            .game
            .log_action(format!("{} has despawned", item.name));
    }

    pub(crate) fn on_player_entered(&mut self, name: String) {
        let Some(room) = &mut self.state.game.room else {
            return;
        };

        room.player_entered(name.clone());

        self.state
            .game
            .log_action(format!("{} entered the room.", name));
    }

    pub(crate) fn on_player_left(&mut self, name: String) {
        if let Some(room) = &mut self.state.game.room {
            room.player_left(&name);
        }

        self.state
            .game
            .log_action(format!("{} left the room.", name));
    }

    pub(crate) fn on_item_taken_by(&mut self, player: String, item_id: String) {
        let Some(room) = &mut self.state.game.room else {
            return;
        };

        let Some(item) = room.take_item(&item_id) else {
            return;
        };

        self.state
            .game
            .log_action(format!("{} took {}.", player, item.name));
    }

    pub(crate) fn on_item_dropped_by(&mut self, player: String, item_id: String) {
        let item = Item::from_manifest(item_id, &self.state.game.manifest);

        let Some(room) = &mut self.state.game.room else {
            return;
        };

        let name = item.name.clone();
        room.spawn_item(item);

        self.state
            .game
            .log_action(format!("{} dropped {}.", player, name));
    }

    pub(crate) fn on_death(&mut self, death_data: DeathData) {
        let is_me = self.state.game.player.is_me(&death_data.player_name);
        let respawn_here = self
            .state
            .game
            .room
            .as_ref()
            .is_some_and(|room| room.name == death_data.respawn_room_id);

        if !is_me && let Some(room) = &mut self.state.game.room {
            if respawn_here {
                room.player_entered(death_data.player_name.clone());
            } else {
                room.player_left(&death_data.player_name);
            }
        }

        let message = match (is_me, respawn_here) {
            (true, true) => "You died and respawned here".to_string(),
            (true, false) => {
                format!("You died and respawned in {}", death_data.respawn_room_id)
            }
            (false, true) => {
                format!("{} died and respawned here", death_data.player_name)
            }
            (false, false) => format!(
                "{} died and respawned in {}",
                death_data.player_name, death_data.respawn_room_id
            ),
        };

        if is_me {
            self.handle_request(ApiRequest::Look(LookCommand));
            self.handle_request(ApiRequest::Status(StatusCommand));
        }

        self.state.game.log_action(message);
    }
}
