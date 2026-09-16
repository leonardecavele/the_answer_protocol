use super::{Overlay, OverlayPayload};
use crate::collections::{Step, move_index};

pub struct FightSummaryState {
    pub selected: usize,
}

impl FightSummaryState {
    pub fn new(count: usize) -> Self {
        Self {
            selected: count.saturating_sub(1),
        }
    }

    pub fn move_selection(&mut self, step: Step, count: usize) {
        self.selected = move_index(self.selected, count, step);
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
