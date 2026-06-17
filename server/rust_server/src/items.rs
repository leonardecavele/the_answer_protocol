

pub type ItemId = u64;

pub struct Item {
    id: ItemId,
    name: String,
    description: String,
}

impl Item {
    pub fn new(id: ItemId, name: String, description: String) -> Self {
        Self { id, name, description }
    }
    pub fn get_id(&self) -> ItemId {
        self.id
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_description(&self) -> &str {
        &self.description
    }
}