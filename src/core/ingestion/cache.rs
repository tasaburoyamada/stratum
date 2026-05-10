use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use std::path::Path;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct IngestionCache {
    pub file_hashes: HashMap<String, String>, // path -> content_hash
}

impl IngestionCache {
    pub fn load(path: &str) -> Result<Self> {
        if Path::new(path).exists() {
            let bytes = std::fs::read(path)?;
            Ok(serde_json::from_slice(&bytes)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_vec(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn check_and_update(&mut self, id: &str, current_hash: &str) -> bool {
        if let Some(h) = self.file_hashes.get(id) {
            if h == current_hash {
                return true; // Unchanged
            }
        }
        self.file_hashes.insert(id.to_string(), current_hash.to_string());
        false // Modified or New
    }
}
