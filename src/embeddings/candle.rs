use crate::embeddings::base::Embedding;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use tokenizers::Tokenizer;
use half::f16;
use std::sync::Arc;
use std::fmt;

pub struct CandleEmbedding {
    model: Arc<BertModel>,
    tokenizer: Arc<Tokenizer>,
    device: Device,
    model_name: String,
}

impl fmt::Debug for CandleEmbedding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CandleEmbedding")
            .field("model_name", &self.model_name)
            .field("device", &self.device)
            .finish()
    }
}

impl CandleEmbedding {
    pub fn new(model_path: &str, tokenizer_path: &str, config_path: &str, device: Option<Device>) -> Result<Self> {
        let device = device.unwrap_or(Device::Cpu); 
        
        // Validate paths before unsafe operations
        if !std::path::Path::new(model_path).exists() {
            return Err(anyhow!("Model file not found at: {}", model_path));
        }
        if !std::path::Path::new(tokenizer_path).exists() {
            return Err(anyhow!("Tokenizer file not found at: {}", tokenizer_path));
        }
        if !std::path::Path::new(config_path).exists() {
            return Err(anyhow!("Config file not found at: {}", config_path));
        }

        // Validate safetensors header to ensure it's not a garbage file
        let mut file = std::fs::File::open(model_path)?;
        use std::io::Read;
        let mut header_buf = [0u8; 8];
        file.read_exact(&mut header_buf).map_err(|_| anyhow!("Failed to read safetensors header"))?;
        let header_size = u64::from_le_bytes(header_buf);
        let file_metadata = std::fs::metadata(model_path)?;
        if header_size == 0 || header_size > file_metadata.len() - 8 {
            return Err(anyhow!("Invalid safetensors header size: {}", header_size));
        }

        let config = std::fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&config)?;
        
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;
        
        // SAFETY: VarBuilder::from_mmaped_safetensors is unsafe because it assumes the 
        // underlying memory map remains valid and the file format is correct.
        // We trust the provided model_path points to a valid safetensors file.
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_path], DTYPE, &device)?
        };
        
        let model = BertModel::load(vb, &config)?;

        Ok(Self {
            model: Arc::new(model),
            tokenizer: Arc::new(tokenizer),
            device,
            model_name: model_path.to_string(),
        })
    }
}

#[async_trait]
impl Embedding for CandleEmbedding {
    async fn get_text_embedding(&self, text: &str) -> Result<Vec<f16>> {
        let tokens = self.tokenizer.encode(text, true)
            .map_err(|e| anyhow!("Tokenizer error: {}", e))?;
        let token_ids = tokens.get_ids();
        let input_ids = Tensor::new(token_ids, &self.device)?.unsqueeze(0)?;
        let token_type_ids = input_ids.zeros_like()?;
        
        let embeddings = self.model.forward(&input_ids, &token_type_ids, None)?;
        
        // Mean pooling
        let (_n_batch, n_tokens, _hidden_size) = embeddings.dims3()?;
        let mean_embedding = (embeddings.sum(1)? / (n_tokens as f64))?;
        let vec: Vec<f32> = mean_embedding.get(0)?.to_vec1()?;
        
        // Convert to f16 for storage consistency
        Ok(vec.into_iter().map(f16::from_f32).collect())
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }
}
