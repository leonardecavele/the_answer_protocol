use crate::app::App;
use crate::collections::SelectableList;
use crate::states::game::{Item, Npc, Room};
use client_api::commands::LookResponse;
use client_api::events::SpawnData;

impl App {
    pub fn on_look(&mut self, response: LookResponse) {
        let manifest = &self.state.game.manifest;
        let players = response.players;

        let room = Room {
            id: response.room.id,
            name: response.room.name,
            description: response.room.description,
            exits: response.room.exits.into(),
            players: SelectableList::with_items(players),
            npcs: response
                .npcs
                .into_iter()
                .map(|id| Npc::from_manifest(id, manifest))
                .collect(),
            items: response
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

    pub fn on_moved(&mut self, direction: String) {
        self.state.game.end_npc_interaction();
        self.state
            .game
            .log_action(format!("You moved {}.", direction));
    }

    pub fn on_npc_spawned(&mut self, spawn: SpawnData) {
        let npc = Npc::from_manifest(spawn.id, &self.state.game.manifest);

        let Some(room) = &mut self.state.game.room else {
            return;
        };

        let name = npc.name.clone();
        room.spawn_npc(npc);

        self.state.game.log_action(format!("{} has respawn", name));
    }

    pub fn on_item_spawned(&mut self, spawn: SpawnData) {
        let item = Item::from_manifest(spawn.id, &self.state.game.manifest);

        let Some(room) = &mut self.state.game.room else {
            return;
        };

        let name = item.name.clone();
        room.spawn_item(item);

        self.state
            .game
            .log_action(format!("{} has been catapulted here", name));
    }

    pub fn on_item_despawned(&mut self, spawn: SpawnData) {
        let Some(room) = &mut self.state.game.room else {
            return;
        };

        let Some(item) = room.take_item(&spawn.id) else {
            return;
        };

        self.state
            .game
            .log_action(format!("{} has despawned", item.name));
    }

    pub fn on_player_entered(&mut self, name: String) {
        let Some(room) = &mut self.state.game.room else {
            return;
        };

        room.player_entered(name.clone());

        self.state
            .game
            .log_action(format!("{} entered the room.", name));
    }

    pub fn on_player_left(&mut self, name: String) {
        if let Some(room) = &mut self.state.game.room {
            room.player_left(&name);
        }

        self.state
            .game
            .log_action(format!("{} left the room.", name));
    }

    pub fn on_item_taken_by(&mut self, player: String, item_id: String) {
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

    pub fn on_item_dropped_by(&mut self, player: String, item_id: String) {
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
}
