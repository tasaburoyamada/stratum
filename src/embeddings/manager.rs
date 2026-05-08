use crate::embeddings::base::Embedding;
use crate::embeddings::candle::CandleEmbedding;
use candle_core::Device;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct EmbeddingManager {
    models: Mutex<HashMap<String, Arc<dyn Embedding>>>,
}

impl Default for EmbeddingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingManager {
    pub fn new() -> Self {
        Self {
            models: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_or_load_candle(
        &self, 
        model_name: &str, 
        model_path: &str, 
        tokenizer_path: &str, 
        config_path: &str,
        device: Option<Device>
    ) -> Result<Arc<dyn Embedding>> {
        let mut models = self.models.lock().map_err(|_| anyhow::anyhow!("ERR_LOCK_POISONED"))?;
        
        if let Some(model) = models.get(model_name) {
            return Ok(model.clone());
        }

        let new_model = Arc::new(CandleEmbedding::new(model_path, tokenizer_path, config_path, device)?);
        models.insert(model_name.to_string(), new_model.clone());
        
        Ok(new_model)
    }
}
