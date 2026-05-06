use crate::extractors::base::MetadataExtractor;
use crate::core::schema::Node;
use crate::llm::LlmClient;
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;

pub struct TitleExtractor {
    llm: Arc<dyn LlmClient>,
}

impl TitleExtractor {
    pub fn new(llm: Arc<dyn LlmClient>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl MetadataExtractor for TitleExtractor {
    async fn extract(&self, mut nodes: Vec<Node>) -> Result<Vec<Node>> {
        for node in &mut nodes {
            if let crate::core::schema::NodeContent::Text(text) = &node.content {
                let prompt = format!("Extract a concise title for the following text. Respond with ONLY the title.\n\n{}", text);
                if let Ok(title) = self.llm.complete(&prompt).await {
                    node.metadata.extra.insert("document_title".to_string(), serde_json::json!(title.trim()));
                }
            }
        }
        Ok(nodes)
    }
}

pub struct SummaryExtractor {
    llm: Arc<dyn LlmClient>,
}

impl SummaryExtractor {
    pub fn new(llm: Arc<dyn LlmClient>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl MetadataExtractor for SummaryExtractor {
    async fn extract(&self, mut nodes: Vec<Node>) -> Result<Vec<Node>> {
        for node in &mut nodes {
            if let crate::core::schema::NodeContent::Text(text) = &node.content {
                let prompt = format!("Summarize the following text into a single, high-density paragraph that captures all key technical details and entities.\n\n{}", text);
                if let Ok(summary) = self.llm.complete(&prompt).await {
                    node.metadata.extra.insert("section_summary".to_string(), serde_json::json!(summary.trim()));
                }
            }
        }
        Ok(nodes)
    }
}
