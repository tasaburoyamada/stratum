use crate::extractors::base::MetadataExtractor;
use crate::core::schema::Node;
use crate::llm::LlmClient;
use crate::core::ingestion::transformation::Transformation;
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;

/// Extractor that evaluates the "Confidence" and "Importance" of information using an LLM.
/// This is a core component for HV-CAD Stratification.
pub struct StratificationExtractor {
    llm: Arc<dyn LlmClient>,
}

impl StratificationExtractor {
    pub fn new(llm: Arc<dyn LlmClient>) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl MetadataExtractor for StratificationExtractor {
    async fn extract(&self, mut nodes: Vec<Node>) -> Result<Vec<Node>> {
        for node in &mut nodes {
            if let crate::core::schema::NodeContent::Text(text) = &node.content {
                let prompt = format!(
                    "Evaluate the following text for two metrics on a scale of 0.0 to 1.0.\n\
                    1. Confidence: How factual, certain, and verifiable is this information? (1.0 = Absolute fact, 0.0 = Pure speculation)\n\
                    2. Importance: How strategically or technically critical is this info for a developer? (1.0 = Critical core logic, 0.0 = Trivia/Noise)\n\n\
                    Respond with ONLY a JSON object: {{\"confidence\": 0.x, \"importance\": 0.y}}\n\n\
                    Text:\n{}",
                    text
                );

                if let Ok(response) = self.llm.complete(&prompt).await {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(response.trim()) {
                        if let Some(c) = val.get("confidence").and_then(|v| v.as_f64()) {
                            node.metadata.confidence = Some(c as f32);
                        }
                        if let Some(i) = val.get("importance").and_then(|v| v.as_f64()) {
                            node.metadata.importance = Some(i as f32);
                        }
                    }
                }
                
                // Set timestamp if missing
                if node.metadata.timestamp.is_none() {
                    node.metadata.timestamp = Some(chrono::Utc::now());
                }
            }
        }
        Ok(nodes)
    }
}

#[async_trait]
impl Transformation for StratificationExtractor {
    async fn transform(&self, nodes: Vec<Node>) -> Result<Vec<Node>> {
        self.extract(nodes).await
    }

    fn hash(&self) -> String {
        "stratification_extractor_v1".to_string()
    }
}

impl std::fmt::Debug for StratificationExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StratificationExtractor").finish()
    }
}
