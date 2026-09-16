use super::{Overlay, OverlayPayload};
use crate::collections::{Step, move_index};

pub struct FightSummaryState {
    pub selected_fight: usize,
    pub selected_player: usize,
}

impl FightSummaryState {
    pub fn new(count: usize) -> Self {
        Self {
            selected_fight: count.saturating_sub(1),
            selected_player: 0,
        }
    }

    pub fn select_fight(&mut self, index: usize) {
        self.selected_fight = index;
        self.selected_player = 0;
    }

    pub fn move_fight_selection(&mut self, step: Step, count: usize) {
        self.select_fight(move_index(self.selected_fight, count, step));
    }

    pub fn move_player_selection(&mut self, step: Step, count: usize) {
        self.selected_player = move_index(self.selected_player, count, step);
    }
}

impl OverlayPayload for FightSummaryState {
    fn extract(overlay: &Overlay) -> Option<&Self> {
        match overlay {
            Overlay::FightSummary(state) => Some(state),
            _ => None,
        }
    }

    fn extract_mut(overlay: &mut Overlay) -> Option<&mut Self> {
        match overlay {
            Overlay::FightSummary(state) => Some(state),
            _ => None,
        }
    }
}
