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

pub struct VlogBiasPostprocessor {
    pub bias: HashMap<String, f32>,
    pub concepts: Vec<String>,
    pub embed_model: Option<Arc<dyn Embedding>>,
}

impl VlogBiasPostprocessor {
    pub fn from_vlog(vlog_content: &str, embed_model: Option<Arc<dyn Embedding>>) -> Self {
        let mut bias = HashMap::new();
        let mut concepts = Vec::new();

        // Parse @BIAS:{P:0.8, M:0.2, ...}
        let re_bias = Regex::new(r"@BIAS:\{([^}]+)\}").unwrap();
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

        // Parse [[CONCEPT]]
        let re_concept = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        for caps in re_concept.captures_iter(vlog_content) {
            concepts.push(caps[1].to_string());
        }

        Self { bias, concepts, embed_model }
    }
}

#[async_trait]
impl NodePostprocessor for VlogBiasPostprocessor {
    async fn postprocess_nodes(&self, nodes: Vec<NodeWithScore>, _query_bundle: &QueryBundle) -> Result<Vec<NodeWithScore>> {
        let mut boosted_nodes = nodes;

        // Pre-calculate concept embeddings if model is available
        let mut concept_embeddings = Vec::new();
        if let Some(model) = &self.embed_model {
            for concept in &self.concepts {
                if let Ok(emb) = model.get_text_embedding(concept).await {
                    concept_embeddings.push(emb);
                }
            }
        }

        for node in &mut boosted_nodes {
            let mut boost = 1.0;
            
            // 1. Semantic Concept Boost
            if !concept_embeddings.is_empty() {
                if let Some(node_emb) = &node.node.embedding {
                    for c_emb in &concept_embeddings {
                        let sim = cosine_similarity(node_emb, c_emb);
                        if sim > 0.7 { // Threshold for semantic relevance
                            boost += (sim - 0.7) * 2.0; // Dynamic boost based on similarity
                        }
                    }
                }
            }

            // 2. Fallback/Complementary Keyword Boost
            if let Ok(content) = node.node.get_content(None) {
                let content_lower = content.to_lowercase();
                for concept in &self.concepts {
                    if content_lower.contains(&concept.to_lowercase()) {
                        boost += 0.1; 
                    }
                }

                // 3. Metadata Bias Boost
                if let Some(genre) = &node.node.metadata.genre {
                    if self.bias.contains_key(genre) {
                        boost += self.bias[genre] * 0.5;
                    }
                }
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
