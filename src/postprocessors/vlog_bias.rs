use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use crate::postprocessors::base::NodePostprocessor;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;

pub struct VlogBiasPostprocessor {
    pub bias: HashMap<String, f32>,
    pub concepts: Vec<String>,
}

impl VlogBiasPostprocessor {
    pub fn from_vlog(vlog_content: &str) -> Self {
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

        Self { bias, concepts }
    }
}

#[async_trait]
impl NodePostprocessor for VlogBiasPostprocessor {
    async fn postprocess_nodes(&self, nodes: Vec<NodeWithScore>, _query_bundle: &QueryBundle) -> Result<Vec<NodeWithScore>> {
        let mut boosted_nodes = nodes;

        for node in &mut boosted_nodes {
            let mut boost = 1.0;
            
            if let Ok(content) = node.node.get_content(None) {
                let content_lower = content.to_lowercase();
                
                // 1. Concept Boost
                for concept in &self.concepts {
                    if content_lower.contains(&concept.to_lowercase()) {
                        boost += 0.2; // 20% boost per matching concept
                    }
                }

                // 2. Metadata Bias Boost (e.g. genre or file type based on vlog priority)
                // This is a placeholder for more complex semantic mapping
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
