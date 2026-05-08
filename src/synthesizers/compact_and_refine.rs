use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use crate::synthesizers::base::{ResponseSynthesizer, DEFAULT_TEXT_QA_PROMPT, DEFAULT_REFINE_PROMPT};
use crate::llm::LlmClient;
use anyhow::Result;
use async_trait::async_trait;
use tiktoken_rs::{cl100k_base, CoreBPE};
use std::sync::Arc;

use crate::core::config::SynthesizerConfig;

pub struct CompactAndRefine {
    pub llm: Arc<dyn LlmClient>,
    pub config: SynthesizerConfig,
    pub bpe: Arc<CoreBPE>,
}

impl CompactAndRefine {
    pub fn new(llm: Arc<dyn LlmClient>, config: SynthesizerConfig) -> Self {
        Self {
            llm,
            config,
            bpe: Arc::new(cl100k_base().unwrap()),
        }
    }

    fn count_tokens(&self, text: &str) -> usize {
        self.bpe.encode_with_special_tokens(text).len()
    }

    fn compact_nodes(&self, nodes: Vec<NodeWithScore>, query_str: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        
        let text_qa_template = self.config.text_qa_template.as_deref().unwrap_or(DEFAULT_TEXT_QA_PROMPT);
        let refine_template = self.config.refine_template.as_deref().unwrap_or(DEFAULT_REFINE_PROMPT);

        // Effective limit for context
        let effective_limit = if self.config.max_tokens > self.config.max_response_tokens {
            self.config.max_tokens - self.config.max_response_tokens
        } else {
            512 // Fallback
        };

        let mut current_len = self.count_tokens(&text_qa_template.replace("{query_str}", query_str));

        for node in nodes {
            if let Ok(text) = node.node.get_content(Some(&self.bpe)) {
                let text_len = self.count_tokens(&text);
                
                // If a single node is larger than the limit, we have to force split it
                // though usually nodes are chunked by splitter earlier.
                if text_len > effective_limit {
                    log::warn!("Single node content ({} tokens) exceeds effective limit ({} tokens)", text_len, effective_limit);
                }

                if current_len + text_len > effective_limit
                    && !current_chunk.is_empty() {
                        chunks.push(current_chunk.join("\n\n"));
                        current_chunk = Vec::new();
                        // Reset current_len with refine template baseline
                        current_len = self.count_tokens(&refine_template.replace("{query_str}", query_str));
                    }
                current_len += text_len;
                current_chunk.push(text);
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk.join("\n\n"));
        }
        chunks
    }
}

#[async_trait]
impl ResponseSynthesizer for CompactAndRefine {
    async fn synthesize(&self, query_bundle: QueryBundle, nodes: Vec<NodeWithScore>) -> Result<String> {
        let compact_chunks = self.compact_nodes(nodes, &query_bundle.query_str);
        
        if compact_chunks.is_empty() {
            return Ok("No relevant context found to answer the query.".to_string());
        }

        let mut response = String::new();
        let text_qa_template = self.config.text_qa_template.as_deref().unwrap_or(DEFAULT_TEXT_QA_PROMPT);
        let refine_template = self.config.refine_template.as_deref().unwrap_or(DEFAULT_REFINE_PROMPT);

        for (i, chunk) in compact_chunks.into_iter().enumerate() {
            if i == 0 {
                let prompt = text_qa_template
                    .replace("{context_str}", &chunk)
                    .replace("{query_str}", &query_bundle.query_str);
                response = self.llm.complete(&prompt).await?;
            } else {
                let prompt = refine_template
                    .replace("{query_str}", &query_bundle.query_str)
                    .replace("{existing_answer}", &response)
                    .replace("{context_msg}", &chunk);
                response = self.llm.complete(&prompt).await?;
            }
        }

        Ok(response)
    }
}
