#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FightPhase {
    #[default]
    Editing,
    AwaitingResult,
    Resolved {
        success: bool,
    },
}

impl FightPhase {
    pub fn submit(&mut self) {
        if let Self::Editing = self {
            *self = Self::AwaitingResult
        }
    }

    pub fn resolve(&mut self, success: bool) {
        if let Self::AwaitingResult = self {
            *self = Self::Resolved { success }
        }
    }

    pub fn reset(&mut self) {
        *self = Self::Editing
    }
}
