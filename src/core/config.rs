use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitterConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub paragraph_separator: String,
    pub separator: String,
}

impl Default for SplitterConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1024,
            chunk_overlap: 200,
            paragraph_separator: "\n\n\n".to_string(),
            separator: " ".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub embed_batch_size: usize,
    pub embedding_dim: usize,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            embed_batch_size: 32,
            embedding_dim: 384, // Default for BERT-small/base
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizerConfig {
    pub max_tokens: usize,           // Total context window size
    pub max_response_tokens: usize,  // Reserved tokens for the answer
    pub text_qa_template: Option<String>,
    pub refine_template: Option<String>,
}

impl Default for SynthesizerConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            max_response_tokens: 512,
            text_qa_template: None,
            refine_template: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub delay_ms: u64,
    pub jitter_ms: u64,
    pub concurrency: usize,
    pub user_agent: String,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            delay_ms: 1000,
            jitter_ms: 500,
            concurrency: 1,
            user_agent: "Stratum-Knowledge-Engine/0.4.0 (Polite-Crawler)".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderConfig {
    pub concurrency: usize,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            concurrency: 4,
        }
    }
}
