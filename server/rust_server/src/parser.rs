use std::fs;
use crate::npc::Npc;

pub struct Parser {
    file_path: String,
}

impl Parser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse_npcs(&self) -> Result<Vec<Npc>, String> {
        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read file '{}': {}", self.file_path, e))?;
        
        let parsed = json::parse(&content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut npcs = Vec::new();
        
        if parsed["npcs"].is_array() {
            for item in parsed["npcs"].members() {
                if let Some(npc) = Npc::new(item) {
                    npcs.push(npc);
                }
            }
        } else {
            return Err("JSON does not contain 'npcs' array".to_string());
        }

        Ok(npcs)
    }
}
