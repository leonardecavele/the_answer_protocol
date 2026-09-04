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
    pub current: u64,
    pub max: u64,
}

impl NpcHealth {
    fn new(current: u64, max: u64) -> Self {
        Self { current, max }
    }

    fn take_damage(&mut self, damage: u32) {
        self.current = self.current.saturating_sub(damage as u64);
    }

    pub fn percent(&self) -> u16 {
        if self.max == 0 {
            return 0;
        }

        (self.current as f64 / self.max as f64 * 100.0) as u16
    }
}

#[derive(Default)]
pub struct FightState {
    phase: FightPhase,
    npc_health: Option<NpcHealth>,
}

impl FightState {
    pub fn start(&mut self, npc_hp: u64, npc_max_hp: u64) {
        self.phase.reset();
        self.npc_health = Some(NpcHealth::new(npc_hp, npc_max_hp));
    }

    pub fn end(&mut self) {
        self.phase.reset();
        self.npc_health = None;
    }

    pub fn submit(&mut self) {
        self.phase.submit();
    }

    pub fn resolve(&mut self, success: bool) {
        self.phase.resolve(success);
    }

    pub fn damage_npc(&mut self, damage: u32) {
        if let Some(health) = &mut self.npc_health {
            health.take_damage(damage);
        }
    }

    pub fn phase(&self) -> FightPhase {
        self.phase
    }

    pub fn npc_health(&self) -> Option<&NpcHealth> {
        self.npc_health.as_ref()
    }
}
