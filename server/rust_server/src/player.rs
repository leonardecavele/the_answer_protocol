pub type PlayerId = u32;
pub type PlayerCount = u32;

pub struct Player {
    name: String, 
    id: PlayerId,
}

impl Player {
    pub fn new(name: String, id: PlayerId) -> Self {
        Self { name, id }
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
}