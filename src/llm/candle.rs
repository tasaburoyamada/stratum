use crate::llm::base::LlmClient;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use candle_core::{Device, Tensor, DType};
use candle_transformers::models::llama::{Llama, LlamaEosToks, Cache};
use tokenizers::Tokenizer;
use std::sync::{Arc, Mutex};
use serde::Deserialize;
use futures::stream::{self, BoxStream, StreamExt};

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
        })
    }
}

#[async_trait]
impl LlmClient for CandleLlm {
    async fn complete(&self, prompt: &str) -> Result<String> {
        let mut stream = self.stream_complete(prompt);
        let mut full_text = String::new();
        while let Some(chunk) = stream.next().await {
            full_text.push_str(&chunk?);
        }
        Ok(full_text)
    }

    fn stream_complete(&self, prompt: &str) -> BoxStream<'static, Result<String>> {
        let prompt = prompt.to_string();
        let device = self.device.clone();
        let tokenizer = self.tokenizer.clone();
        let model = self.model.clone();
        let cache = self.cache.clone();

        let s = stream::unfold(
            (0, vec![], true), // (index, current_tokens, first_run)
            move |(index, mut tokens, first_run)| {
                let model = model.clone();
                let cache = cache.clone();
                let tokenizer = tokenizer.clone();
                let device = device.clone();
                let prompt = prompt.clone();

                async move {
                    if index >= 512 { return None; }

                    let mut model = model.lock().ok()?;
                    let mut cache = cache.lock().ok()?;

                    if first_run {
                        let t = tokenizer.encode(prompt, true).ok()?;
                        tokens = t.get_ids().to_vec();
                    }

                    let context_size = if !first_run { 1 } else { tokens.len() };
                    let start_pos = tokens.len().saturating_sub(context_size);
                    
                    let input = Tensor::new(&tokens[start_pos..], &device).ok()?.unsqueeze(0).ok()?;
                    let logits = model.forward(&input, start_pos, &mut cache).ok()?;
                    let logits = logits.squeeze(0).ok()?;
                    let logits = logits.get(logits.dim(0).ok()? - 1).ok()?;
                    
                    // Simple sampling (greedy for now)
                    let pr: Vec<f32> = logits.to_vec1::<f32>().ok()?;
                    let token = pr.iter().enumerate()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                        .map(|(idx, _)| idx as u32)?;

                    tokens.push(token);

                    if token == 2 { return None; } // EOS

                    let text = tokenizer.id_to_token(token)?
                        .replace(' ', " ")
                        .replace("<0x0A>", "\n");

                    Some((Ok(text), (index + 1, tokens, false)))
                }
            }
        );

        s.boxed()
    }

    fn clone_box(&self) -> Box<dyn LlmClient> {
        Box::new(Self {
            device: self.device.clone(),
            tokenizer: self.tokenizer.clone(),
            model: self.model.clone(),
            cache: self.cache.clone(),
        })
    }
}
