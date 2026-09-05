use crate::{constants::LootType, player::PlayerId};
use json::JsonValue;
use tracing::warn;

pub type Questid = String;

#[derive(Clone)]
pub struct Loot {
    pub qty: u32,
    pub chance: f32,
    pub loot_type: LootType,
}

impl Loot {
    pub fn to_json(&self) -> JsonValue {
        json::object! {
            "qty" => self.qty,
            "chance" => self.chance,
            "type" => self.loot_type.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Quest {
    name: String,
    description: String,
    loots: Vec<Loot>,
    nb_steps: u32,
}

impl Quest {
    pub fn new(json: &JsonValue) -> Option<Self> {
        let name = json["name"].as_str()?;
        let description = json["description"].as_str()?;

        let nb_steps = json["nb_steps"].as_u32()?;
        if nb_steps == 0 {
            warn!("Quest '{}' has invalid nb_steps: cannot be 0", name);
            return None;
        }

        let mut loots = Vec::new();
        if json["loots"].is_object() {
            for (key, val) in json["loots"].entries() {
                let loot_type = LootType::from_string(key)?;
                let qty = val["qty"].as_u32()?;
                let chance = val["chance"].as_f64()? as f32;
                if qty == 0 || !(0.0..=100.0).contains(&chance) {
                    return None;
                }
                loots.push(Loot {
                    qty,
                    chance,
                    loot_type,
                });
            }
        }

        Some(Self {
            name: name.to_string(),
            description: description.to_string(),
            loots,
            nb_steps,
        })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_nb_steps(&self) -> u32 {
        self.nb_steps
    }

    pub fn get_json_loots(&self) -> JsonValue {
        let vec: Vec<JsonValue> = self.loots.iter().map(|loot| loot.to_json()).collect();
        JsonValue::Array(vec)
    }

    pub fn get_loots(&self) -> &Vec<Loot> {
        &self.loots
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuestInstance {
    player: PlayerId,
    quest: Questid,
    current_step: u32,
}

impl QuestInstance {
    pub fn new(player: PlayerId, quest: Questid) -> Self {
        Self {
            player,
            quest,
            current_step: 0,
        }
    }

    pub fn new_with_step(
        player: PlayerId,
        quest: Questid,
        current_step: u32,
    ) -> Self {
        Self {
            player,
            quest,
            current_step,
        }
    }

    pub fn get_player(&self) -> PlayerId {
        self.player
    }

    pub fn get_quest_name(&self) -> Questid {
        self.quest.clone()
    }

    pub fn get_state(&self) -> &str {
        "in progress"
    }

    pub fn get_current_step(&self) -> u32 {
        self.current_step
    }

    pub fn set_current_step(&mut self, current_step: u32) {
        self.current_step = current_step;
    }
}
