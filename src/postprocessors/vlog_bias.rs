use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use crate::postprocessors::base::NodePostprocessor;
use crate::embeddings::base::Embedding;
use crate::vector_stores::utils::cosine_similarity;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

static RE_BIAS: OnceLock<Regex> = OnceLock::new();
static RE_CONCEPT: OnceLock<Regex> = OnceLock::new();
static RE_CTX: OnceLock<Regex> = OnceLock::new();

/// HV-CAD Dynamic Bias Postprocessor
/// 
/// Implements three layers of bias:
/// L1: Surface/Keyword bias (Metadata matching)
/// L2: Semantic/Concept bias (Embedding similarity)
/// L3: Intent/Logical bias (Context-aware filtering)
/// Plus Stratification: Time, Confidence, and Importance.
pub struct VlogBiasPostprocessor {
    pub bias: HashMap<String, f32>,       // L1: Metadata/Genre bias
    pub concepts: Vec<String>,           // L2: Semantic concepts
    pub intents: Vec<String>,            // L3: High-level intents/goals
    pub embed_model: Option<Arc<dyn Embedding>>,
    pub reference_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl VlogBiasPostprocessor {
    pub fn from_vlog(vlog_content: &str, embed_model: Option<Arc<dyn Embedding>>) -> Self {
        let mut bias = HashMap::new();
        let mut concepts = Vec::new();
        let mut intents = Vec::new();

        // Parse L1 Bias: @BIAS:{P:0.8, M:0.2, ...}
        let re_bias = RE_BIAS.get_or_init(|| Regex::new(r"@BIAS:\{([^}]+)\}").unwrap());
        if let Some(caps) = re_bias.captures(vlog_content) {
            for pair in caps[1].split(',') {
                let kv: Vec<&str> = pair.split(':').collect();
                if kv.len() == 2 {
                    if let Ok(val) = kv[1].trim().parse::<f32>() {
                        bias.insert(kv[0].trim().to_string(), val);
                    }
                }
            }
        }

        // Parse L2 Concepts: [[CONCEPT]]
        let re_concept = RE_CONCEPT.get_or_init(|| Regex::new(r"\[\[([^\]]+)\]\]").unwrap());
        for caps in re_concept.captures_iter(vlog_content) {
            concepts.push(caps[1].to_string());
        }

        // Parse L3 Context/Intents: @CTX:[DOM:..|SUB:..|GOAL:..]
        let re_ctx = RE_CTX.get_or_init(|| Regex::new(r"@CTX:\[([^\]]+)\]").unwrap());
        if let Some(caps) = re_ctx.captures(vlog_content) {
            for part in caps[1].split('|') {
                let kv: Vec<&str> = part.split(':').collect();
                if kv.len() == 2 {
                    intents.push(kv[1].trim().to_string());
                }
            }
        }

        Self { bias, concepts, intents, embed_model, reference_time: None }
    }

    pub fn with_reference_time(mut self, time: chrono::DateTime<chrono::Utc>) -> Self {
        self.reference_time = Some(time);
        self
    }
}

#[async_trait]
impl NodePostprocessor for VlogBiasPostprocessor {
    async fn postprocess_nodes(&self, nodes: Vec<NodeWithScore>, _query_bundle: &QueryBundle) -> Result<Vec<NodeWithScore>> {
        let mut boosted_nodes = nodes;

        // Pre-calculate L2 concept embeddings if model is available
        let mut concept_embeddings = Vec::new();
        if let Some(model) = &self.embed_model {
            for concept in &self.concepts {
                if let Ok(emb) = model.get_text_embedding(concept).await {
                    concept_embeddings.push(emb);
                }
            }
        }

        let ref_time = self.reference_time.unwrap_or_else(chrono::Utc::now);

        for node in &mut boosted_nodes {
            let mut boost = 1.0;
            
            // 1. HV-CAD L2: Semantic Concept Boost (Embedding similarity)
            if !concept_embeddings.is_empty() {
                if let Some(node_emb) = &node.node.embedding {
                    for c_emb in &concept_embeddings {
                        let sim = cosine_similarity(node_emb, c_emb);
                        if sim > 0.7 { 
                            boost += (sim - 0.7) * 4.0; // Significant boost for semantic matches
                        }
                    }
                }
            }

            // HV-CAD: Even if content is Purged, we can boost via Metadata/Intents
            let content_lower = match node.node.get_content(None) {
                Ok(content) => Some(content.to_lowercase()),
                Err(_) => None, // Node is Purged and no BPE available
            };

            // 2. HV-CAD L2 Fallback: Keyword Concept Boost
            for concept in &self.concepts {
                let concept_lower = concept.to_lowercase();
                // Check in content (if available)
                if let Some(cl) = &content_lower {
                    if cl.contains(&concept_lower) {
                        boost += 0.25;
                        continue;
                    }
                }
                // Check in extra metadata for keyword tags
                if let Some(tags) = node.node.metadata.extra.get("tags").and_then(|v| v.as_array()) {
                    if tags.iter().any(|t| t.as_str().map(|s| s.to_lowercase() == concept_lower).unwrap_or(false)) {
                        boost += 0.25;
                    }
                }
            }

            // 3. HV-CAD L3: Intent/Context Boost (Highest priority)
            for intent in &self.intents {
                let intent_lower = intent.to_lowercase();
                if let Some(cl) = &content_lower {
                    if cl.contains(&intent_lower) {
                        boost += 0.6;
                        continue;
                    }
                }
                // Logical check: Metadata intent alignment
                if let Some(node_intent) = node.node.metadata.extra.get("intent").and_then(|v| v.as_str()) {
                    if node_intent.to_lowercase() == intent_lower {
                        boost += 0.8; // Metadata match is stronger than keyword match
                    }
                }
            }

            // 4. HV-CAD L1: Metadata Bias Boost
            if let Some(genre) = &node.node.metadata.genre {
                if let Some(&val) = self.bias.get(genre) {
                    boost += val * 0.5;
                }
            }

            // 5. Stratification: Confidence and Importance
            if let Some(confidence) = node.node.metadata.confidence {
                boost += confidence * 0.4; // 0.0-1.0 range
            }
            if let Some(importance) = node.node.metadata.importance {
                boost += importance * 0.5; // 0.0-1.0 range
            }

            // 6. Stratification: Temporal Decay (Freshness)
            if let Some(timestamp) = node.node.metadata.timestamp {
                let age_days = (ref_time - timestamp).num_days() as f32;
                // Decay: 1.0 at day 0, 0.5 at day 30 (approx 0.98^age)
                // Use a softer decay for very old items to avoid zeroing out scores
                let decay = 0.98_f32.powf(age_days.max(0.0).min(180.0));
                boost *= decay;
            }

            if let Some(score) = node.score {
                node.score = Some(score * boost);
            }
        }

        // Re-sort after boosting
        boosted_nodes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(boosted_nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::schema::Node;
    use crate::core::query_bundle::QueryBundle;
    use crate::core::schema::NodeWithScore;
    use chrono::{Utc, Duration};

    #[test]
    fn test_vlog_bias_parsing() {
        let vlog_content = r#"
        @BIAS:{SciFi:1.5, Fantasy:-0.5}
        [[AI]] [[Rust]]
        @CTX:[DOM:PROJECT|SUB:DEV|GOAL:STABLE]
        "#;

        let postprocessor = VlogBiasPostprocessor::from_vlog(vlog_content, None);
        
        assert_eq!(postprocessor.bias.get("SciFi"), Some(&1.5));
        assert!(postprocessor.concepts.contains(&"AI".to_string()));
        assert!(postprocessor.intents.contains(&"STABLE".to_string()));
    }

    #[tokio::test]
    async fn test_stratification_boost() {
        let postprocessor = VlogBiasPostprocessor::from_vlog("", None);
        
        let mut node_old = Node::new_text("Old important info".to_string());
        node_old.metadata.timestamp = Some(Utc::now() - Duration::days(30));
        node_old.metadata.importance = Some(1.0);

        let mut node_new = Node::new_text("New info".to_string());
        node_new.metadata.timestamp = Some(Utc::now());
        node_new.metadata.importance = Some(0.1);

        let nodes = vec![
            NodeWithScore { node: node_old, score: Some(1.0) },
            NodeWithScore { node: node_new, score: Some(1.0) },
        ];

        let boosted = postprocessor.postprocess_nodes(nodes, &QueryBundle::new("query".to_string())).await.unwrap();
        
        // Old node has high importance but decay. New node has low importance but no decay.
        // Node 0 (old) boost: (1.0 + 1.0*0.5) * 0.98^30 = 1.5 * 0.54 = 0.81
        // Node 1 (new) boost: (1.0 + 0.1*0.5) * 1.0 = 1.05
        assert_eq!(boosted[0].node.get_content(None).unwrap(), "New info");
    }

    #[tokio::test]
    async fn test_purged_node_boost() {
        let mut node = Node::new_text("This is about Rust.".to_string());
        node.metadata.extra.insert("tags".to_string(), serde_json::json!(["Rust"]));
        node.purge_text();

        let postprocessor = VlogBiasPostprocessor::from_vlog("[[Rust]]", None);
        let nodes = vec![NodeWithScore { node, score: Some(1.0) }];

        let result = postprocessor.postprocess_nodes(nodes, &QueryBundle::new("query".to_string())).await.unwrap();
        assert!(result[0].score.unwrap() > 1.0);
    }
}
