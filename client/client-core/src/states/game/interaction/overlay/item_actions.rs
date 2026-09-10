use super::{Overlay, OverlayPayload};
use crate::collections::SelectableList;

pub enum ItemLocation {
    Room,
    Inventory,
}

pub struct ItemActionsState {
    pub item_id: String,
    pub actions: SelectableList<String>,
}

impl ItemActionsState {
    pub const TAKE: &'static str = "TAKE";
    pub const USE: &'static str = "USE";
    pub const DROP: &'static str = "DROP";
    pub const VIEW: &'static str = "VIEW";
    pub const CANCEL: &'static str = "CANCEL";

    pub fn new(item_id: String, useable: bool, location: ItemLocation) -> Self {
        let reach = match location {
            ItemLocation::Room => Self::TAKE,
            ItemLocation::Inventory => Self::DROP,
        };

        let mut actions = vec![reach.to_string()];

        if useable {
            actions.push(Self::USE.to_string());
        }

        actions.push(Self::VIEW.to_string());
        actions.push(Self::CANCEL.to_string());

        let mut actions = SelectableList::with_items(actions);
        actions.select_index(0);

        Self { item_id, actions }
    }
}

impl OverlayPayload for ItemActionsState {
    fn extract(overlay: &Overlay) -> Option<&Self> {
        match overlay {
            Overlay::ItemActions(state) => Some(state),
            _ => None,
        }
    }

    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self> {
        match overlay {
            Overlay::ItemActions(state) => Some(state),
            _ => None,
        }
    }
}
