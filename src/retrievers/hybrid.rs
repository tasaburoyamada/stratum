use crate::core::schema::NodeWithScore;
use crate::core::query_bundle::QueryBundle;
use crate::retrievers::base::Retriever;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;

pub enum HybridMode {
    WeightedSum,
    ReciprocalRankFusion,
}

pub struct HybridRetriever {
    pub retrievers: Vec<Arc<dyn Retriever>>,
    pub mode: HybridMode,
    pub weights: Vec<f32>,
}

impl HybridRetriever {
    pub fn new(retrievers: Vec<Arc<dyn Retriever>>, mode: HybridMode, weights: Option<Vec<f32>>) -> Self {
        let weights = weights.unwrap_or_else(|| vec![1.0; retrievers.len()]);
        Self {
            retrievers,
            mode,
            weights,
        }
    }
}

#[async_trait]
impl Retriever for HybridRetriever {
    async fn retrieve(&self, query_bundle: QueryBundle) -> Result<Vec<NodeWithScore>> {
        let mut all_results = Vec::new();

        // 1. Fetch from all retrievers
        for retriever in &self.retrievers {
            let res = retriever.retrieve(query_bundle.clone()).await?;
            all_results.push(res);
        }

        match self.mode {
            HybridMode::WeightedSum => {
                let mut node_map: HashMap<String, (NodeWithScore, f32)> = HashMap::new();
                
                for (i, results) in all_results.into_iter().enumerate() {
                    let weight = self.weights.get(i).cloned().unwrap_or(1.0);
                    for r in results {
                        let id = r.node.id_.clone();
                        let score = r.score.unwrap_or(0.0) * weight;
                        
                        let entry = node_map.entry(id).or_insert((r, 0.0));
                        entry.1 += score;
                    }
                }

                let mut final_results: Vec<NodeWithScore> = node_map.into_values()
                    .map(|(mut r, score)| {
                        r.score = Some(score);
                        r
                    })
                    .collect();
                
                final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                Ok(final_results)
            }
            HybridMode::ReciprocalRankFusion => {
                let mut node_map: HashMap<String, (NodeWithScore, f32)> = HashMap::new();
                let k = 60.0; // Standard constant for RRF

                for results in all_results {
                    for (rank, r) in results.into_iter().enumerate() {
                        let id = r.node.id_.clone();
                        let rrf_score = 1.0 / (k + (rank as f32) + 1.0);
                        
                        let entry = node_map.entry(id).or_insert((r, 0.0));
                        entry.1 += rrf_score;
                    }
                }

                let mut final_results: Vec<NodeWithScore> = node_map.into_values()
                    .map(|(mut r, score)| {
                        r.score = Some(score);
                        r
                    })
                    .collect();
                
                final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                Ok(final_results)
            }
        }
    }
}
