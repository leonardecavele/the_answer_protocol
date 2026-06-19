use crate::constantes::HARDCODED_PLAYER_ROOM;
use crate::groups::GroupId;
use crate::inventory::Inventory;
use crate::items::ItemId;
use crate::npc::Npc;
use crate::room::RoomName;
use std::collections::{HashMap, HashSet};

pub type PlayerId = u32;
pub type PlayerCount = u32;

pub struct Player {
    name: String,
    id: PlayerId,
    group_id: Option<GroupId>,
    inventory: Inventory,
    current_room: String,
    dialogs_index: HashMap<String, usize>,
}

impl Player {
    pub fn new(name: String, id: PlayerId) -> Self {
        Self {
            name,
            id,
            group_id: None,
            inventory: Inventory::new(),
            current_room: HARDCODED_PLAYER_ROOM.to_string(),
            dialogs_index: HashMap::new(),
        }
    }
    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_id(&self) -> PlayerId {
        self.id
    }
    pub fn get_group_id(&self) -> Option<GroupId> {
        self.group_id
    }
    pub fn get_items(&self) -> &HashSet<ItemId> {
        &self.inventory.get_items()
    }
    pub fn add_item(&mut self, item_id: ItemId) {
        self.inventory.add_item(item_id);
    }
    pub fn remove_item(&mut self, item_id: ItemId) {
        self.inventory.remove_item(item_id);
    }
    pub fn get_current_room(&self) -> &str {
        &self.current_room
    }
    pub fn move_to_room(&mut self, room: &RoomName) {
        self.current_room = room.clone();
    }
    pub fn get_dialog_index_for_npc(&self, npc_name: &str) -> usize {
        return self.dialogs_index.get(npc_name).copied().unwrap_or(0);
    }
    pub fn talk_with(&mut self, npc: &Npc) -> String {
        let dialog_index = self.get_dialog_index_for_npc(npc.get_name().as_str());
        let mut dialog = npc.get_dialog(dialog_index);

        if dialog.is_some() {
            self.dialogs_index.insert(npc.get_name(), dialog_index + 1);
        } else {
            let _ = self.dialogs_index.insert(npc.get_name(), 0);
            // dialogs list return to index 0 after finish so we use the first dialog
            // if none is found (because of a too high index)
            // it is checked sooner if the npc does not have dialog or no
            dialog = npc.get_dialog(0);
        }

        return dialog.unwrap().to_string();
    }
}
