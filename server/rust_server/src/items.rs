pub type ItemId = u64;

#[derive(Clone)]
pub struct Item {
    id: ItemId,
    name: String,
    description: String,
}

impl Item {
    pub fn new(id: ItemId, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
        }
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

    pub fn parse_item(item: &str) -> Option<(ItemId, String)> {
        let parts: Vec<_> = item.split('.').collect();
        if !(parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()) {
            return None;
        }
        Some((parts[0].parse::<ItemId>().ok()?, parts[1].to_string()))
    }
}
