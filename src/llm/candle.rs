use crate::llm::base::LlmClient;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use candle_core::{Device, Tensor, DType};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::llama::{Llama, LlamaEosToks, Cache};
use tokenizers::Tokenizer;
use std::sync::{Arc, Mutex};
use serde::Deserialize;

#[derive(Deserialize)]
struct LlamaConfig {
    hidden_size: usize,
    intermediate_size: usize,
    vocab_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: usize,
    #[serde(default)]
    use_flash_attn: bool,
    rms_norm_eps: f64,
    #[serde(default = "default_rope_theta")]
    rope_theta: f32,
    bos_token_id: Option<u32>,
    eos_token_id: Option<u32>,
    max_position_embeddings: usize,
}

fn default_rope_theta() -> f32 { 10000.0 }

impl From<LlamaConfig> for candle_transformers::models::llama::Config {
    fn from(c: LlamaConfig) -> Self {
        Self {
            hidden_size: c.hidden_size,
            intermediate_size: c.intermediate_size,
            vocab_size: c.vocab_size,
            num_hidden_layers: c.num_hidden_layers,
            num_attention_heads: c.num_attention_heads,
            num_key_value_heads: c.num_key_value_heads,
            use_flash_attn: c.use_flash_attn,
            rms_norm_eps: c.rms_norm_eps,
            rope_theta: c.rope_theta,
            bos_token_id: c.bos_token_id,
            eos_token_id: c.eos_token_id.map(LlamaEosToks::Single),
            rope_scaling: None,
            max_position_embeddings: c.max_position_embeddings,
            tie_word_embeddings: false,
        }
    }
}

pub struct CandleLlm {
    device: Device,
    tokenizer: Tokenizer,
    model: Arc<Mutex<Llama>>,
    cache: Arc<Mutex<Cache>>,
    model_name: String,
}

impl CandleLlm {
    pub fn new(
        model_path: &str,
        tokenizer_path: &str,
        config_path: &str,
        device: Option<Device>,
    ) -> Result<Self> {
        let device = device.unwrap_or(Device::Cpu);
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;
        
        let config_str = std::fs::read_to_string(config_path)?;
        let config_local: LlamaConfig = serde_json::from_str(&config_str)?;
        let config: candle_transformers::models::llama::Config = config_local.into();

        let vb = unsafe {
            candle_nn::VarBuilder::from_mmaped_safetensors(&[model_path], DType::F32, &device)?
        };

        let llama = Llama::load(vb, &config)?;
        let cache = Cache::new(true, DType::F32, &config, &device)?;

        Ok(Self {
            device,
            tokenizer,
            model: Arc::new(Mutex::new(llama)),
            cache: Arc::new(Mutex::new(cache)),
            model_name: "llama".to_string(),
        })
    }
}

#[async_trait]
impl LlmClient for CandleLlm {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let mut model = self.model.lock().map_err(|_| anyhow!("Model lock poisoned"))?;
        let mut cache = self.cache.lock().map_err(|_| anyhow!("Cache lock poisoned"))?;
        
        let tokens = self.tokenizer.encode(prompt, true)
            .map_err(|e| anyhow!("Tokenizer error: {}", e))?;
        let prompt_tokens = tokens.get_ids();
        
        let mut tokens = prompt_tokens.to_vec();
        let mut logits_processor = LogitsProcessor::new(299792458, Some(0.7), None);

        let mut generated_text = String::new();
        let max_gen_len = 512;

        for index in 0..max_gen_len {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let input = Tensor::new(&tokens[start_pos..], &self.device)?.unsqueeze(0)?;
            let logits = model.forward(&input, start_pos, &mut cache)?;
            let logits = logits.squeeze(0)?;
            let logits = logits.get(logits.dim(0)? - 1)?;
            let token = logits_processor.sample(&logits)?;
            tokens.push(token);

            if let Some(t) = self.tokenizer.id_to_token(token) {
                let s = t.replace(' ', " ").replace("<0x0A>", "\n");
                generated_text.push_str(&s);
            }

            if token == 2 { 
                break;
            }
        }

        Ok(generated_text)
    }
}
