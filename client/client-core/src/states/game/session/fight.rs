use client_api::events::FightEndData;
use std::collections::VecDeque;

const MAX_HISTORY: usize = 10;

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
    fn submit(&mut self) {
        if let Self::Editing = self {
            *self = Self::AwaitingResult
        }
    }

    fn resolve(&mut self, success: bool) {
        if let Self::AwaitingResult = self {
            *self = Self::Resolved { success }
        }
    }

    fn reset(&mut self) {
        *self = Self::Editing
    }
}

pub struct NpcHealth {
    pub current: u16,
    pub max: u16,
}

impl NpcHealth {
    fn new(current: u16, max: u16) -> Self {
        Self { current, max }
    }

    fn set_hp(&mut self, hp: u16) {
        self.current = hp;
    }

    fn take_damage(&mut self, damage: u16) {
        self.current = self.current.saturating_sub(damage);
    }

    pub fn percent(&self) -> u16 {
        if self.max == 0 {
            return 0;
        }

        (f64::from(self.current) / f64::from(self.max) * 100.0) as u16
    }
}

#[derive(Default)]
pub struct FightState {
    history: VecDeque<FightEndData>,
    phase: FightPhase,
    npc_health: Option<NpcHealth>,
}

impl FightState {
    pub fn start(&mut self, npc_hp: u16, npc_max_hp: u16) {
        self.phase.reset();
        self.npc_health = Some(NpcHealth::new(npc_hp, npc_max_hp));
    }

    pub fn end(&mut self) {
        self.phase.reset();
        self.npc_health = None;
    }

    pub fn push_history(&mut self, data: FightEndData) {
        self.history.push_back(data);
        if self.history.len() > MAX_HISTORY {
            self.history.pop_front();
        }
    }

    pub fn submit(&mut self) {
        self.phase.submit();
    }

    pub fn resolve(&mut self, success: bool) {
        self.phase.resolve(success);
    }

    pub fn damage_npc(&mut self, damage: u16) {
        if let Some(health) = &mut self.npc_health {
            health.take_damage(damage);
        }
    }

    pub fn set_npc_hp(&mut self, hp: u16) {
        if let Some(health) = &mut self.npc_health {
            health.set_hp(hp);
        }
    }

    pub fn phase(&self) -> FightPhase {
        self.phase
    }

    pub fn npc_health(&self) -> Option<&NpcHealth> {
        self.npc_health.as_ref()
    }

    pub fn history(&self) -> &VecDeque<FightEndData> {
        &self.history
    }
}
