pub struct UiState {
    pub notifications: Vec<String>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }
}
