pub struct FightState {
    pub submitted: bool,
    pub success: Option<bool>,
}

impl FightState {
    pub fn new() -> Self {
        Self {
            submitted: false,
            success: None,
        }
    }

    pub fn reset(&mut self) {
        self.submitted = false;
        self.success = None;
    }
}

impl Default for FightState {
    fn default() -> Self {
        Self::new()
    }
}
