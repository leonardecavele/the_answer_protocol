use crate::constantes::LOOT;
use json::JsonValue;

pub type Questid = String;

#[derive(Clone)]
pub struct Loot {
    pub qty: u32,
    pub chance: f32,
    pub loot_type: LOOT,
}

#[derive(Clone)]
pub struct Quest {
    id: Questid,
    name: String,
    description: String,
    loots: Vec<Loot>,
}

impl Quest {
    pub fn new(json: &JsonValue) -> Option<Self> {
        let name = json["name"].as_str()?;
        let id = json["id"].as_str().unwrap_or(name).to_string();
        let description = json["description"].as_str()?;
        
        let mut loots = Vec::new();
        if json["loots"].is_object() {
            for (key, val) in json["loots"].entries() {
                let loot_type = LOOT::from_string(key)?;
                let qty = val["qty"].as_u32().unwrap_or(1);
                let chance = val["chance"].as_f64().unwrap_or(100.0) as f32;
                loots.push(Loot { qty, chance, loot_type });
            }
        }
        
        Some(Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            loots,
        })
    }

    pub fn get_id(&self) -> &Questid {
        &self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_loots(&self) -> &Vec<Loot> {
        &self.loots
    }
}
