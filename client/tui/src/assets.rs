
use ratatui_image::picker::Picker;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;


#[derive(Debug, Deserialize, Serialize)]
pub struct MappingRule {
    pub room: Option<String>,
    pub npc: Option<String>,
    pub image: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub mappings: Vec<MappingRule>,
    #[serde(default)]
    pub display_names: std::collections::HashMap<String, String>,
}

pub struct AssetManager {
    manifest: Manifest,
    image_cache: HashMap<String, ratatui_image::protocol::StatefulProtocol>,
    picker: Picker,
    base_dir: std::path::PathBuf,
}

impl AssetManager {
    pub fn new() -> Self {
        // Find the root directory and assets directory from global config
        let root_dir = crate::config::ROOT_DIR
            .get()
            .cloned()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let assets_dir = crate::config::ASSETS_DIR
            .get()
            .cloned()
            .unwrap_or_else(|| std::path::PathBuf::from("assets"));

        let manifest_path = root_dir.join("manifest.json");
        let manifest: Manifest = if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path).unwrap_or_default();
            match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to parse manifest.json: {:?}", e);
                    Manifest {
                        mappings: vec![],
                        display_names: HashMap::new(),
                    }
                }
            }
        } else {
            log::error!("manifest.json not found at {:?}", manifest_path);
            Manifest {
                mappings: vec![],
                display_names: HashMap::new(),
            }
        };

        let picker = Picker::halfblocks();

        Self {
            manifest,
            image_cache: HashMap::new(),
            picker,
            base_dir: root_dir,
        }
    }

    pub fn get_display_name(&self, id: &str) -> String {
        self.manifest
            .display_names
            .get(id)
            .cloned()
            .unwrap_or_else(|| id.to_string())
    }

    pub fn get_image_for_context(
        &mut self,
        room_name: &str,
        npcs: &[String],
    ) -> Option<&mut ratatui_image::protocol::StatefulProtocol> {
        let mut selected_path = None;
        let mut best_score = -1;

        log::info!("Matching room: {:?} with npcs: {:?}", room_name, npcs);

        for rule in &self.manifest.mappings {
            let mut match_score = 0;
            let mut possible = true;

            if let Some(r) = &rule.room {
                if r == room_name {
                    match_score += 1;
                } else if r != "default" {
                    possible = false;
                }
            }

            if let Some(n) = &rule.npc {
                if npcs.contains(n) {
                    match_score += 1;
                } else {
                    possible = false;
                }
            }

            if possible && match_score > best_score {
                best_score = match_score;
                selected_path = Some(rule.image.clone());
            }
        }

        log::info!("Selected path from manifest: {:?}", selected_path);
        let path = selected_path?;
        let full_path = self.base_dir.join(&path);

        log::info!("Trying to load image: {:?}", full_path);

        if !self.image_cache.contains_key(&path) {
            let load_result = image::ImageReader::open(&full_path)
                .map_err(|e| e.to_string())
                .and_then(|r| r.with_guessed_format().map_err(|e| e.to_string()))
                .and_then(|r| r.decode().map_err(|e| e.to_string()));

            match load_result {
                Ok(img) => {
                    log::info!("Successfully loaded image: {:?}", full_path);
                    let dyn_img = self.picker.new_resize_protocol(img);
                    self.image_cache.insert(path.clone(), dyn_img);
                }
                Err(e) => {
                    log::error!("Failed to load image {:?}: {}", full_path, e);
                    return None;
                }
            }
        }

        self.image_cache.get_mut(&path)
    }
}
