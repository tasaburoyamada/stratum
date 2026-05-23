use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use crate::retrievers::base::Retriever;
use crate::postprocessors::vlog_bias::VlogBiasPostprocessor;
use crate::postprocessors::base::NodePostprocessor;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HybridMode {
    WeightedSum,
    ReciprocalRankFusion,
}

pub struct HybridRetriever {
    pub retrievers: Vec<Arc<dyn Retriever>>,
    pub mode: HybridMode,
    pub bias_postprocessor: Option<Arc<VlogBiasPostprocessor>>,
}

impl HybridRetriever {
    pub fn new(retrievers: Vec<Arc<dyn Retriever>>, mode: HybridMode, bias_postprocessor: Option<Arc<VlogBiasPostprocessor>>) -> Self {
        Self {
            retrievers,
            mode,
            bias_postprocessor,
        }
    }

    fn weighted_sum(&self, all_results: Vec<Vec<NodeWithScore>>) -> Vec<NodeWithScore> {
        use std::collections::HashMap;
        let mut node_map: HashMap<String, NodeWithScore> = HashMap::new();

        for nodes in all_results {
            for node in nodes {
                let entry = node_map.entry(node.node.id_.to_string()).or_insert_with(|| {
                    let mut n = node.clone();
                    n.score = Some(0.0);
                    n
                });
                if let Some(s) = node.score {
                    *entry.score.as_mut().unwrap() += s;
                }
            }
        }

        let mut final_nodes: Vec<NodeWithScore> = node_map.into_values().collect();
        final_nodes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        final_nodes
    }

    fn rrf(&self, all_results: Vec<Vec<NodeWithScore>>) -> Vec<NodeWithScore> {
        use std::collections::HashMap;
        let mut node_map: HashMap<String, (NodeWithScore, f32)> = HashMap::new();
        let k = 60.0; // RRF constant

        for nodes in all_results {
            for (rank, node) in nodes.iter().enumerate() {
                let entry = node_map.entry(node.node.id_.to_string()).or_insert_with(|| {
                    (node.clone(), 0.0)
                });
                entry.1 += 1.0 / (k + (rank + 1) as f32);
            }
        }

        let mut final_nodes: Vec<NodeWithScore> = node_map.into_values()
            .map(|(mut node, rrf_score)| {
                node.score = Some(rrf_score);
                node
            })
            .collect();
        final_nodes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        final_nodes
    }
}

#[async_trait]
impl Retriever for HybridRetriever {
    async fn retrieve(&self, query_bundle: QueryBundle) -> Result<Vec<NodeWithScore>> {
        let mut all_results = Vec::new();
        for retriever in &self.retrievers {
            let nodes = retriever.retrieve(query_bundle.clone()).await?;
            all_results.push(nodes);
        }

        let merged_nodes = match self.mode {
            HybridMode::WeightedSum => self.weighted_sum(all_results),
            HybridMode::ReciprocalRankFusion => self.rrf(all_results),
        };

        if let Some(post) = &self.bias_postprocessor {
            post.postprocess_nodes(merged_nodes, &query_bundle).await
        } else {
            Ok(merged_nodes)
        }
    }
}
